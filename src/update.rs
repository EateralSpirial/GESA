//! Signed immutable releases and a restart-safe activation transaction.
//!
//! A launcher is deliberately separate from the replaceable core. Updates affect
//! subsequent invocations; already-running commands retain their executable.
use crate::{auth, config::Config};
use anyhow::{anyhow, bail, ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, VerifyingKey};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

const MAX_ARTIFACT: u64 = 64 * 1024 * 1024;
const MAX_METADATA: u64 = 512 * 1024;
const MAX_HEALTH_OUTPUT: u64 = 16 * 1024;
const HEALTH_TIMEOUT: Duration = Duration::from_secs(5);
const STATE_FILE: &str = "update-state.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub schema: u32,
    pub sequence: u64,
    pub version: String,
    pub channel: String,
    pub target: String,
    pub artifact: String,
    pub sha256: String,
    pub size: u64,
    pub expires_at: u64,
    pub protocol: u32,
    pub contract_epoch: u32,
    pub supports_contracts: Vec<u32>,
    pub iteration: Value,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    payload: String,
    signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Slot {
    bootstrap: bool,
    sha256: String,
    size: u64,
    sequence: u64,
    version: String,
    contract_epoch: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Pending {
    from: Slot,
    candidate: Slot,
    phase: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    schema: u32,
    bootstrap: Slot,
    active: Slot,
    previous: Option<Slot>,
    pending: Option<Pending>,
    highest_sequence: u64,
    highest_manifest_sha256: String,
    observed_at: u64,
    last_event: String,
    #[serde(default)]
    quarantined: Vec<Slot>,
}

fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub fn target() -> String {
    format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
}

fn binary_name() -> &'static str {
    if cfg!(windows) {
        "gesa-core.exe"
    } else {
        "gesa-core"
    }
}

fn valid_hash(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn no_symlink(path: &Path) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) => ensure!(
            !meta.file_type().is_symlink(),
            "symlink paths are not accepted"
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}

fn private_dir(path: &Path) -> Result<()> {
    crate::config::ensure_private_dir(path)
}

fn private_options() -> OpenOptions {
    let options = OpenOptions::new();
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        let mut options = options;
        options.mode(0o600);
        options
    }
    #[cfg(not(unix))]
    options
}

fn sync_dir(path: &Path) -> Result<()> {
    #[cfg(unix)]
    File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}

fn lock(home: &Path) -> Result<File> {
    private_dir(home)?;
    let path = home.join("update.lock");
    no_symlink(&path)?;
    let file = private_options()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(path)?;
    file.try_lock_exclusive()
        .map_err(|_| anyhow!("another GESA update is in progress"))?;
    Ok(file)
}

fn load_state(home: &Path) -> Result<State> {
    private_dir(home)?;
    let path = home.join(STATE_FILE);
    let bytes = crate::config::read_private_file(&path, MAX_METADATA)
        .context("initialize GESA before using the updater; preserve damaged state for recovery")?;
    let state: State = serde_json::from_slice(&bytes)
        .context("update state is damaged; preserve it for recovery")?;
    ensure!(state.schema == 1, "unsupported update state schema");
    for slot in [&state.bootstrap, &state.active]
        .into_iter()
        .chain(state.previous.iter())
        .chain(state.pending.iter().flat_map(|p| [&p.from, &p.candidate]))
        .chain(state.quarantined.iter())
    {
        ensure!(
            valid_hash(&slot.sha256) && slot.size > 0 && slot.size <= MAX_ARTIFACT,
            "invalid installed release metadata"
        );
        ensure!(slot.contract_epoch > 0, "invalid installed contract epoch");
        semver::Version::parse(&slot.version).context("invalid installed version")?;
    }
    ensure!(
        state.bootstrap.bootstrap && state.bootstrap.sequence == 0,
        "invalid bootstrap metadata"
    );
    ensure!(
        state.highest_sequence >= state.active.sequence,
        "invalid update high-watermark"
    );
    if state.highest_sequence > 0 {
        ensure!(
            valid_hash(&state.highest_manifest_sha256),
            "invalid manifest high-watermark"
        );
    }
    Ok(state)
}

fn save_state(home: &Path, state: &State) -> Result<()> {
    let mut bytes = serde_json::to_vec_pretty(state)?;
    ensure!(
        (bytes.len() as u64) < MAX_METADATA,
        "update state exceeds size limit"
    );
    bytes.push(b'\n');
    crate::config::write_private_file(&home.join(STATE_FILE), &bytes)
}

fn slot_path(home: &Path, slot: &Slot) -> Result<PathBuf> {
    ensure!(valid_hash(&slot.sha256), "invalid release digest");
    let root = if slot.bootstrap {
        home.join("bootstrap")
    } else {
        home.join("releases").join(&slot.sha256)
    };
    private_dir(home)?;
    private_dir(&home.join("releases"))?;
    private_dir(&root)?;
    let path = root.join(binary_name());
    no_symlink(&path)?;
    Ok(path)
}

fn digest_file(path: &Path, limit: u64) -> Result<(String, u64)> {
    no_symlink(path)?;
    let file = File::open(path)?;
    ensure!(
        file.metadata()?.is_file(),
        "release artifact must be a regular file"
    );
    ensure!(
        file.metadata()?.len() <= limit,
        "release artifact is oversized"
    );
    let mut reader = file.take(limit + 1);
    let mut hasher = Sha256::new();
    let mut total = 0;
    let mut buffer = [0u8; 32 * 1024];
    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total += count as u64;
        ensure!(total <= limit, "release artifact is oversized");
        hasher.update(&buffer[..count]);
    }
    Ok((hex::encode(hasher.finalize()), total))
}

fn verify_slot(home: &Path, slot: &Slot) -> Result<PathBuf> {
    let path = slot_path(home, slot)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let metadata = fs::metadata(&path)?;
        ensure!(
            metadata.permissions().mode() & 0o077 == 0
                && metadata.uid() == fs::metadata(home)?.uid(),
            "installed release requires private owner-only permissions"
        );
    }
    let (hash, size) = digest_file(&path, MAX_ARTIFACT)?;
    ensure!(
        hash == slot.sha256 && size == slot.size,
        "installed release integrity check failed"
    );
    Ok(path)
}

fn executable(path: &Path) -> Result<()> {
    crate::config::harden_new_executable(path)
}

/// Copy the currently executing trusted core once. A later release never replaces it.
pub fn install_bootstrap(home: &Path) -> Result<()> {
    let _lock = lock(home)?;
    if home.join(STATE_FILE).exists() {
        let state = load_state(home)?;
        verify_slot(home, &state.bootstrap)?;
        return Ok(());
    }
    private_dir(&home.join("bootstrap"))?;
    private_dir(&home.join("releases"))?;
    let source = std::env::current_exe()?;
    let (hash, size) = digest_file(&source, MAX_ARTIFACT)?;
    let slot = Slot {
        bootstrap: true,
        sha256: hash,
        size,
        sequence: 0,
        version: option_env!("GESA_RELEASE_VERSION")
            .unwrap_or(env!("CARGO_PKG_VERSION"))
            .to_owned(),
        contract_epoch: 1,
    };
    let dest = slot_path(home, &slot)?;
    if dest.exists() {
        verify_slot(home, &slot).context("an incomplete bootstrap differs from this core")?;
    } else {
        let temp = home
            .join("bootstrap")
            .join(format!(".bootstrap-{}.tmp", rand::random::<u64>()));
        fs::copy(&source, &temp)?;
        executable(&temp)?;
        File::open(&temp)?.sync_all()?;
        fs::rename(&temp, &dest)?;
        sync_dir(&home.join("bootstrap"))?;
    }
    save_state(
        home,
        &State {
            schema: 1,
            bootstrap: slot.clone(),
            active: slot,
            previous: None,
            pending: None,
            highest_sequence: 0,
            highest_manifest_sha256: String::new(),
            observed_at: now()?,
            last_event: "initialized".into(),
            quarantined: Vec::new(),
        },
    )
}

fn bounded_response(response: reqwest::blocking::Response, limit: u64) -> Result<Vec<u8>> {
    ensure!(
        response.status().is_success(),
        "center request was rejected ({})",
        response.status().as_u16()
    );
    if let Some(length) = response.content_length() {
        ensure!(length <= limit, "center response exceeds size limit");
    }
    let mut bytes = Vec::new();
    response
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .context("could not read center response")?;
    ensure!(
        bytes.len() as u64 <= limit,
        "center response exceeds size limit"
    );
    Ok(bytes)
}

fn authenticate_release(bytes: &[u8], config: &Config) -> Result<(Release, String)> {
    ensure!(
        bytes.len() as u64 <= MAX_METADATA,
        "release metadata exceeds size limit"
    );
    let envelope: Envelope = serde_json::from_slice(bytes).context("invalid release envelope")?;
    let payload = STANDARD
        .decode(&envelope.payload)
        .context("invalid release payload encoding")?;
    ensure!(
        payload.len() as u64 <= MAX_METADATA / 2,
        "release payload exceeds size limit"
    );
    let key_bytes: [u8; 32] = hex::decode(&config.release_public_key)
        .context("invalid trusted release key")?
        .try_into()
        .map_err(|_| anyhow!("invalid trusted release key length"))?;
    let key = VerifyingKey::from_bytes(&key_bytes).context("invalid trusted release key")?;
    let signature = Signature::from_slice(
        &hex::decode(&envelope.signature).context("invalid release signature encoding")?,
    )
    .context("invalid release signature length")?;
    key.verify_strict(&payload, &signature)
        .context("release signature verification failed")?;
    let release: Release =
        serde_json::from_slice(&payload).context("invalid signed release payload")?;
    ensure!(
        release.schema == 1 && release.protocol == 1,
        "unsupported release protocol"
    );
    ensure!(
        release.target == target(),
        "release target does not match this machine"
    );
    ensure!(
        release.channel == config.channel,
        "release channel mismatch"
    );
    let manifest_hash = hex::encode(Sha256::digest(&payload));
    ensure!(
        release.size > 0 && release.size <= MAX_ARTIFACT,
        "invalid artifact size"
    );
    ensure!(valid_hash(&release.sha256), "invalid artifact digest");
    semver::Version::parse(&release.version).context("invalid release version")?;
    ensure!(
        release
            .iteration
            .get("artifact_sha256")
            .and_then(Value::as_str)
            == Some(release.sha256.as_str()),
        "iteration evidence is bound to another artifact"
    );
    crate::protocol::validate(
        &release.iteration,
        release.sequence,
        release.contract_epoch,
        &release.supports_contracts,
    )?;
    artifact_url(config, &release.artifact)?;
    Ok((release, manifest_hash))
}

fn decode_release(
    bytes: &[u8],
    config: &Config,
    state: &State,
    timestamp: u64,
) -> Result<(Release, String)> {
    let (release, manifest_hash) = authenticate_release(bytes, config)?;
    ensure!(
        release.sequence > 0 && release.sequence >= state.highest_sequence,
        "release rollback attack detected"
    );
    if release.sequence == state.highest_sequence {
        ensure!(
            manifest_hash == state.highest_manifest_sha256,
            "release sequence was reused with different metadata"
        );
    }
    ensure!(
        release.expires_at > timestamp.max(state.observed_at),
        "release metadata has expired"
    );
    ensure!(
        semver::Version::parse(&release.version)? >= semver::Version::parse(&state.active.version)?,
        "release version downgrade rejected"
    );
    ensure!(
        release.contract_epoch >= state.active.contract_epoch
            && release.contract_epoch <= state.active.contract_epoch.saturating_add(1),
        "contract epoch must advance at most one generation"
    );
    Ok((release, manifest_hash))
}

fn receipt_path(home: &Path, slot: &Slot) -> Result<PathBuf> {
    ensure!(
        !slot.bootstrap,
        "bootstrap has a local installation receipt in update state"
    );
    Ok(slot_path(home, slot)?
        .parent()
        .context("invalid slot path")?
        .join(format!("release-{}.json", slot.sequence)))
}

fn save_receipt(home: &Path, slot: &Slot, exact_envelope: &[u8]) -> Result<()> {
    let path = receipt_path(home, slot)?;
    if path.exists() {
        let existing = crate::config::read_private_file(&path, MAX_METADATA)?;
        ensure!(
            existing == exact_envelope,
            "immutable release receipt differs from signed metadata"
        );
    } else {
        crate::config::write_private_file(&path, exact_envelope)?;
    }
    Ok(())
}

fn verify_receipt(home: &Path, config: &Config, slot: &Slot) -> Result<()> {
    if slot.bootstrap {
        return Ok(());
    }
    let exact_envelope =
        crate::config::read_private_file(&receipt_path(home, slot)?, MAX_METADATA)?;
    let (release, _) = authenticate_release(&exact_envelope, config)?;
    ensure!(
        release_slot(&release) == *slot,
        "signed receipt does not match installed slot metadata"
    );
    // Installed/retained releases remain verifiable offline after publication metadata expires.
    // Expiry and monotonic sequence checks apply when accepting new remote updates.
    Ok(())
}

fn artifact_url(config: &Config, path: &str) -> Result<url::Url> {
    ensure!(
        path.starts_with('/') && !path.starts_with("//") && !path.contains('\\'),
        "artifact must use an absolute same-origin URL path"
    );
    ensure!(
        !path.split('/').any(|part| part == ".." || part == "."),
        "artifact path traversal rejected"
    );
    let base = config.endpoint("/")?;
    let url = base.join(path).context("invalid artifact path")?;
    ensure!(
        url.origin() == base.origin()
            && url.username().is_empty()
            && url.password().is_none()
            && url.fragment().is_none()
            && url.query().is_none(),
        "artifact URL must stay on the configured center origin"
    );
    Ok(url)
}

fn authorized_get(
    home: &Path,
    config: &Config,
    url: url::Url,
    token: &mut String,
) -> Result<reqwest::blocking::Response> {
    let client = config.client()?;
    let response = client
        .get(url.clone())
        .bearer_auth(&*token)
        .send()
        .map_err(|_| anyhow!("center request failed"))?;
    if response.status() != reqwest::StatusCode::UNAUTHORIZED {
        return Ok(response);
    }
    // A center restart or token revocation can invalidate an otherwise unexpired cache.
    // One refresh only; 403 and repeated 401 are returned to the caller.
    drop(response);
    let session = auth::login(home, config)?;
    use zeroize::Zeroize;
    token.zeroize();
    *token = session.access_token.clone();
    client
        .get(url)
        .bearer_auth(&*token)
        .send()
        .map_err(|_| anyhow!("center retry failed"))
}

fn fetch_release(
    home: &Path,
    config: &Config,
    state: &State,
) -> Result<(Release, String, String, Vec<u8>)> {
    config.validate()?;
    let mut token = auth::token(home, config)?;
    let mut url = config.endpoint("/v1/releases/latest")?;
    url.query_pairs_mut()
        .append_pair("channel", &config.channel)
        .append_pair("target", &target());
    let response = authorized_get(home, config, url, &mut token)?;
    let bytes = bounded_response(response, MAX_METADATA)?;
    let (release, manifest_hash) = decode_release(&bytes, config, state, now()?)?;
    Ok((release, manifest_hash, token, bytes))
}

fn release_slot(release: &Release) -> Slot {
    Slot {
        bootstrap: false,
        sha256: release.sha256.clone(),
        size: release.size,
        sequence: release.sequence,
        version: release.version.clone(),
        contract_epoch: release.contract_epoch,
    }
}

fn stage(
    home: &Path,
    config: &Config,
    release: &Release,
    token: &mut String,
    exact_envelope: &[u8],
) -> Result<Slot> {
    let slot = release_slot(release);
    private_dir(&home.join("releases"))?;
    let root = home.join("releases").join(&release.sha256);
    private_dir(&root)?;
    let dest = slot_path(home, &slot)?;
    if dest.exists() {
        verify_slot(home, &slot)?;
        save_receipt(home, &slot, exact_envelope)?;
        return Ok(slot);
    }
    let url = artifact_url(config, &release.artifact)?;
    let response = authorized_get(home, config, url, token)?;
    let bytes = bounded_response(response, release.size)?;
    ensure!(
        bytes.len() as u64 == release.size,
        "artifact length does not match signed metadata"
    );
    ensure!(
        hex::encode(Sha256::digest(&bytes)) == release.sha256,
        "artifact digest does not match signed metadata"
    );
    let temp = root.join(format!(".download-{}.tmp", rand::random::<u64>()));
    let result = (|| {
        let mut file = private_options().write(true).create_new(true).open(&temp)?;
        file.write_all(&bytes)?;
        file.sync_all()?;
        drop(file);
        executable(&temp)?;
        fs::rename(&temp, &dest)?;
        sync_dir(&root)?;
        verify_slot(home, &slot)?;
        save_receipt(home, &slot, exact_envelope)?;
        Ok(slot)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temp);
    }
    result
}

fn health(home: &Path, slot: &Slot) -> Result<()> {
    let path = verify_slot(home, slot)?;
    let mut command = Command::new(path);
    command
        .arg("__health")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .env_clear()
        .env("GESA_HOME", home);
    // Windows' loader may require these variables; no credential-bearing environment is inherited.
    for name in ["SystemRoot", "WINDIR", "TEMP", "TMP"] {
        if let Some(value) = std::env::var_os(name) {
            command.env(name, value);
        }
    }
    let mut child = command
        .spawn()
        .context("candidate health process could not start")?;
    let stdout = child
        .stdout
        .take()
        .context("candidate health output unavailable")?;
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let result = stdout
            .take(MAX_HEALTH_OUTPUT + 1)
            .read_to_end(&mut bytes)
            .map(|_| bytes);
        let _ = tx.send(result);
    });
    let deadline = Instant::now() + HEALTH_TIMEOUT;
    let mut output = None;
    let mut exited = None;
    let result = loop {
        if output.is_none() {
            match rx.try_recv() {
                Ok(Ok(bytes)) if bytes.len() as u64 <= MAX_HEALTH_OUTPUT => output = Some(bytes),
                Ok(_) => break Err(anyhow!("candidate health output is invalid or oversized")),
                Err(mpsc::TryRecvError::Empty) => (),
                Err(mpsc::TryRecvError::Disconnected) => {
                    break Err(anyhow!("candidate health output unavailable"))
                }
            }
        }
        if exited.is_none() {
            match child.try_wait() {
                Ok(status) => exited = status,
                Err(_) => break Err(anyhow!("candidate health process could not be observed")),
            }
        }
        if let (Some(status), Some(bytes)) = (exited, output.as_ref()) {
            if !status.success() {
                break Err(anyhow!("candidate health check failed"));
            }
            let value: Value = match serde_json::from_slice(bytes) {
                Ok(value) => value,
                Err(_) => break Err(anyhow!("candidate health output is not JSON")),
            };
            if value.get("healthy") != Some(&Value::Bool(true))
                || value.get("protocol").and_then(Value::as_u64) != Some(1)
            {
                break Err(anyhow!("candidate health protocol check failed"));
            }
            if value.get("version").and_then(Value::as_str) != Some(slot.version.as_str())
                || value.get("target").and_then(Value::as_str) != Some(target().as_str())
            {
                break Err(anyhow!(
                    "candidate health identity does not match signed release"
                ));
            }
            break Ok(());
        }
        if Instant::now() >= deadline {
            break Err(anyhow!("candidate health check timed out"));
        }
        thread::sleep(Duration::from_millis(10));
    };
    if exited.is_none() {
        let _ = child.kill();
    }
    let _ = child.wait();
    result
}

fn recover_state(home: &Path, state: &mut State) -> Result<bool> {
    if let Some(pending) = state.pending.clone() {
        ensure!(
            pending.phase == "prepared" || pending.phase == "activated",
            "invalid activation journal phase"
        );
        let fallback = [&pending.from]
            .into_iter()
            .chain(state.previous.iter())
            .chain([&state.bootstrap])
            .find(|slot| verify_slot(home, slot).is_ok())
            .cloned()
            .context("all recovery copies for interrupted update failed integrity checks")?;
        state.active = fallback;
        state.pending = None;
        state.last_event = "interrupted_update_recovered".into();
        save_state(home, state)?;
        return Ok(true);
    }
    if verify_slot(home, &state.active).is_err() {
        let fallback = state
            .previous
            .as_ref()
            .filter(|s| verify_slot(home, s).is_ok())
            .or_else(|| {
                if verify_slot(home, &state.bootstrap).is_ok() {
                    Some(&state.bootstrap)
                } else {
                    None
                }
            })
            .cloned()
            .context("active, previous, and bootstrap recovery copies failed integrity checks")?;
        state.active = fallback;
        state.last_event = "integrity_failure_recovered".into();
        save_state(home, state)?;
        return Ok(true);
    }
    Ok(false)
}

pub fn check(home: &Path, config: &Config) -> Result<Value> {
    let state = load_state(home)?;
    ensure!(
        state.pending.is_none(),
        "an interrupted activation requires gesa recover"
    );
    let (release, _, _, _) = fetch_release(home, config, &state)?;
    let blocked = state
        .quarantined
        .iter()
        .any(|slot| slot.sha256 == release.sha256);
    Ok(
        json!({"available": !blocked && (release.sequence != state.active.sequence || release.sha256 != state.active.sha256), "blocked":blocked,
        "release": release, "active": state.active, "highest_sequence": state.highest_sequence}),
    )
}

pub fn apply(home: &Path, config: &Config) -> Result<Value> {
    let _lock = lock(home)?;
    let mut state = load_state(home)?;
    recover_state(home, &mut state)?;
    let (release, manifest_hash, mut token, exact_envelope) = fetch_release(home, config, &state)?;
    if release.sequence == state.active.sequence && release.sha256 == state.active.sha256 {
        return Ok(json!({"updated":false,"active":state.active,"reason":"already_current"}));
    }
    ensure!(
        !state
            .quarantined
            .iter()
            .any(|slot| slot.sha256 == release.sha256),
        "candidate is quarantined after a failed health check; publish a corrected artifact"
    );
    // Persist the signed high-watermark before downloading or executing any candidate.
    state.highest_sequence = release.sequence;
    state.highest_manifest_sha256 = manifest_hash;
    state.observed_at = state.observed_at.max(now()?);
    state.last_event = "release_verified".into();
    save_state(home, &state)?;
    let candidate = match stage(home, config, &release, &mut token, &exact_envelope) {
        Ok(candidate) => candidate,
        Err(error) => {
            state.last_event = "staging_failed".into();
            save_state(home, &state)?;
            return Err(error);
        }
    };
    if health(home, &candidate).is_err() {
        state.quarantined.push(candidate);
        state.last_event = "pre_activation_health_failed".into();
        save_state(home, &state)?;
        bail!(
            "candidate failed pre-activation health and was quarantined; active release unchanged"
        );
    }
    let previous = state.active.clone();
    state.pending = Some(Pending {
        from: previous.clone(),
        candidate: candidate.clone(),
        phase: "prepared".into(),
    });
    state.last_event = "activation_prepared".into();
    save_state(home, &state)?;
    state.active = candidate.clone();
    state.pending.as_mut().expect("pending set above").phase = "activated".into();
    state.last_event = "activation_pending_health".into();
    save_state(home, &state)?;
    if health(home, &candidate).is_err() {
        state.active = previous;
        state.pending = None;
        state.quarantined.push(candidate);
        state.last_event = "health_failure_rolled_back".into();
        save_state(home, &state)?;
        bail!("candidate failed post-activation health; previous release restored");
    }
    state.previous = Some(previous);
    state.pending = None;
    state.last_event = "update_committed".into();
    save_state(home, &state)?;
    Ok(
        json!({"updated":true,"active":state.active,"previous":state.previous,"highest_sequence":state.highest_sequence}),
    )
}

pub fn rollback(home: &Path) -> Result<Value> {
    let _lock = lock(home)?;
    let mut state = load_state(home)?;
    recover_state(home, &mut state)?;
    let previous = state
        .previous
        .clone()
        .context("no previous release is available")?;
    ensure!(
        previous.sha256 != state.active.sha256,
        "previous release is already active"
    );
    health(home, &previous)?;
    // Keep previous pointing at the known recovery version. A rollback is not a toggle.
    if !state
        .quarantined
        .iter()
        .any(|slot| slot.sha256 == state.active.sha256)
    {
        state.quarantined.push(state.active.clone());
    }
    state.active = previous;
    state.last_event = "operator_rollback".into();
    save_state(home, &state)?;
    Ok(json!({"rolled_back":true,"active":state.active,"highest_sequence":state.highest_sequence}))
}

pub fn recover(home: &Path) -> Result<Value> {
    let _lock = lock(home)?;
    let mut state = load_state(home)?;
    let recovered = recover_state(home, &mut state)?;
    Ok(
        json!({"recovered":recovered,"active":state.active,"highest_sequence":state.highest_sequence}),
    )
}

pub fn status(home: &Path) -> Result<Value> {
    let state = load_state(home)?;
    Ok(serde_json::to_value(state)?)
}

pub fn verify(home: &Path) -> Result<Value> {
    let state = load_state(home)?;
    let config = Config::load(home)?;
    verify_slot(home, &state.active)?;
    verify_receipt(home, &config, &state.active)?;
    verify_slot(home, &state.bootstrap)?;
    if let Some(previous) = &state.previous {
        verify_slot(home, previous)?;
        verify_receipt(home, &config, previous)?;
    }
    ensure!(
        state.pending.is_none(),
        "activation journal requires recovery"
    );
    Ok(json!({"verified":true,"active":state.active,"highest_sequence":state.highest_sequence}))
}

/// Stable launcher entrypoint. Never retries a command that has begun execution.
pub fn launcher_main() -> Result<i32> {
    let home = crate::config::home_dir()?;
    let mut state = load_state(&home)?;
    let needs_recovery = state.pending.is_some() || verify_slot(&home, &state.active).is_err();
    if needs_recovery {
        match lock(&home) {
            Ok(_lock) => {
                state = load_state(&home)?;
                if recover_state(&home, &mut state)? {
                    eprintln!("GESA: recovered the last verified executable");
                }
            }
            Err(_) if state.pending.is_some() => {
                // An updater may still be running. Keep new commands on the committed old core.
                state.active = state.pending.as_ref().expect("checked above").from.clone();
            }
            Err(error) => return Err(error),
        }
    }
    let path = verify_slot(&home, &state.active)?;
    let mut command = Command::new(path);
    command.args(std::env::args_os().skip(1));
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        Err(command.exec()).context("verified GESA core could not start")
    }
    #[cfg(not(unix))]
    {
        let exit = command
            .status()
            .context("verified GESA core could not start")?;
        Ok(exit.code().unwrap_or(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::{Signer, SigningKey};

    struct TestDir(PathBuf);
    impl TestDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "gesa-update-test-{}-{}",
                std::process::id(),
                rand::random::<u64>()
            ));
            private_dir(&path).unwrap();
            Self(path)
        }
    }
    impl Drop for TestDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn key() -> SigningKey {
        SigningKey::from_bytes(&[19; 32])
    }
    fn config() -> Config {
        Config {
            schema: 1,
            center_url: "https://example.test".into(),
            account: "test".into(),
            release_public_key: hex::encode(key().verifying_key().to_bytes()),
            channel: "stable".into(),
            allow_http_loopback: false,
            credential_store: "os".into(),
        }
    }
    fn initial_state() -> State {
        let slot = Slot {
            bootstrap: true,
            sha256: "a".repeat(64),
            size: 10,
            sequence: 0,
            version: "0.1.0".into(),
            contract_epoch: 1,
        };
        State {
            schema: 1,
            bootstrap: slot.clone(),
            active: slot,
            previous: None,
            pending: None,
            highest_sequence: 0,
            highest_manifest_sha256: String::new(),
            observed_at: 1000,
            last_event: "initialized".into(),
            quarantined: Vec::new(),
        }
    }
    fn release() -> Release {
        Release {
            schema: 1,
            sequence: 1,
            version: "0.2.0".into(),
            channel: "stable".into(),
            target: target(),
            artifact: "/v1/artifacts/test".into(),
            sha256: "b".repeat(64),
            size: 10,
            expires_at: 2000,
            protocol: 1,
            contract_epoch: 1,
            supports_contracts: vec![1],
            iteration: json!({"candidate_id":"issue-1", "source_revision":"a".repeat(40),
                "artifact_sha256":"b".repeat(64), "change_summary":"fix", "risk":"low",
                "tests":[{"name":"smoke", "passed":true, "evidence_sha256":"c".repeat(64)}],
                "approval":{"decision":"approved", "actor":"policy-v1", "policy":"automatic-low-risk"},
                "rollback":"restore previous core", "migration":{"kind":"none","reversible":true},
                "protected_core_changed":false}),
        }
    }
    fn signed(item: &Release) -> Vec<u8> {
        let payload = serde_json::to_vec(item).unwrap();
        serde_json::to_vec(&json!({"payload":STANDARD.encode(&payload), "signature":hex::encode(key().sign(&payload).to_bytes())})).unwrap()
    }
    fn accepted(item: &Release) -> bool {
        decode_release(&signed(item), &config(), &initial_state(), 1500).is_ok()
    }
    fn write_slot(home: &Path, bootstrap: bool, sequence: u64, bytes: &[u8]) -> Slot {
        let slot = Slot {
            bootstrap,
            sha256: hex::encode(Sha256::digest(bytes)),
            size: bytes.len() as u64,
            sequence,
            version: if sequence == 0 { "0.1.0" } else { "0.2.0" }.into(),
            contract_epoch: 1,
        };
        let path = slot_path(home, &slot).unwrap();
        let mut f = private_options()
            .write(true)
            .create_new(true)
            .open(&path)
            .unwrap();
        f.write_all(bytes).unwrap();
        f.sync_all().unwrap();
        executable(&path).unwrap();
        slot
    }

    #[test]
    fn accepts_exact_signed_payload() {
        assert!(accepted(&release()));
    }

    #[test]
    fn tampered_payload_and_wrong_trust_key_are_rejected() {
        let bytes = signed(&release());
        let mut envelope: Value = serde_json::from_slice(&bytes).unwrap();
        let mut changed = release();
        changed.version = "0.3.0".into();
        envelope["payload"] = STANDARD
            .encode(serde_json::to_vec(&changed).unwrap())
            .into();
        assert!(decode_release(
            &serde_json::to_vec(&envelope).unwrap(),
            &config(),
            &initial_state(),
            1500
        )
        .is_err());
        let mut c = config();
        c.release_public_key =
            hex::encode(SigningKey::from_bytes(&[21; 32]).verifying_key().to_bytes());
        assert!(decode_release(&bytes, &c, &initial_state(), 1500).is_err());
    }

    #[test]
    fn expired_future_observed_clock_and_downgrade_are_rejected() {
        let mut r = release();
        r.expires_at = 1500;
        assert!(!accepted(&r));
        let mut state = initial_state();
        state.observed_at = 3000;
        assert!(decode_release(&signed(&release()), &config(), &state, 1500).is_err());
        let mut r = release();
        r.version = "0.0.9".into();
        assert!(!accepted(&r));
    }

    #[test]
    fn signed_target_channel_protocol_and_size_are_enforced() {
        let mut r = release();
        r.target = "wrong-platform".into();
        assert!(!accepted(&r));
        let mut r = release();
        r.channel = "canary".into();
        assert!(!accepted(&r));
        let mut r = release();
        r.protocol = 2;
        assert!(!accepted(&r));
        let mut r = release();
        r.schema = 2;
        assert!(!accepted(&r));
        let mut r = release();
        r.size = MAX_ARTIFACT + 1;
        assert!(!accepted(&r));
        let mut r = release();
        r.size = 0;
        assert!(!accepted(&r));
    }

    #[test]
    fn contract_jump_and_evidence_artifact_mismatch_are_rejected() {
        let mut r = release();
        r.contract_epoch = 3;
        r.supports_contracts = vec![1, 2, 3];
        assert!(!accepted(&r));
        let mut r = release();
        r.contract_epoch = 2;
        r.supports_contracts = vec![1, 2];
        assert!(accepted(&r));
        let mut r = release();
        r.iteration["artifact_sha256"] = "c".repeat(64).into();
        assert!(!accepted(&r));
        let mut r = release();
        r.iteration["tests"][0]["passed"] = false.into();
        assert!(!accepted(&r));
    }

    #[test]
    fn high_watermark_detects_rollback_and_sequence_equivocation() {
        let r = release();
        let (_, hash) = decode_release(&signed(&r), &config(), &initial_state(), 1500).unwrap();
        let mut state = initial_state();
        state.highest_sequence = 1;
        state.highest_manifest_sha256 = hash;
        assert!(decode_release(&signed(&r), &config(), &state, 1500).is_ok());
        let mut changed = r.clone();
        changed.expires_at = 2001;
        assert!(decode_release(&signed(&changed), &config(), &state, 1500).is_err());
        state.highest_sequence = 2;
        assert!(decode_release(&signed(&r), &config(), &state, 1500).is_err());
    }

    #[test]
    fn installed_receipt_reverifies_offline_even_after_metadata_expires() {
        let dir = TestDir::new();
        let slot = write_slot(&dir.0, false, 1, b"candidate data");
        let mut r = release();
        r.sha256 = slot.sha256.clone();
        r.size = slot.size;
        r.iteration["artifact_sha256"] = slot.sha256.clone().into();
        r.expires_at = 1;
        let exact_envelope = signed(&r);
        save_receipt(&dir.0, &slot, &exact_envelope).unwrap();
        assert_eq!(
            fs::read(receipt_path(&dir.0, &slot).unwrap()).unwrap(),
            exact_envelope
        );
        verify_receipt(&dir.0, &config(), &slot).unwrap();
        assert!(decode_release(&exact_envelope, &config(), &initial_state(), 1500).is_err());
        let mut wrong_slot = slot.clone();
        wrong_slot.version = "9.0.0".into();
        assert!(verify_receipt(&dir.0, &config(), &wrong_slot).is_err());
        let mut tampered: Value = serde_json::from_slice(&exact_envelope).unwrap();
        tampered["signature"] = "00".repeat(64).into();
        crate::config::write_private_file(
            &receipt_path(&dir.0, &slot).unwrap(),
            &serde_json::to_vec(&tampered).unwrap(),
        )
        .unwrap();
        assert!(verify_receipt(&dir.0, &config(), &slot).is_err());
    }

    #[test]
    fn artifact_cannot_escape_origin_or_embed_credentials() {
        for path in [
            "https://evil.test/a",
            "//evil.test/a",
            "\\evil.test\\a",
            "/../secret",
            "/a/./b",
            "/a#b",
            "/a?secret=b",
        ] {
            assert!(artifact_url(&config(), path).is_err(), "accepted {path}");
        }
        assert_eq!(
            artifact_url(&config(), "/v1/artifacts/hash")
                .unwrap()
                .as_str(),
            "https://example.test/v1/artifacts/hash"
        );
    }

    #[test]
    fn interrupted_activation_restores_previous_and_preserves_high_watermark() {
        let dir = TestDir::new();
        let bootstrap = write_slot(&dir.0, true, 0, b"bootstrap data");
        let candidate = write_slot(&dir.0, false, 1, b"candidate data");
        let mut state = initial_state();
        state.bootstrap = bootstrap.clone();
        state.active = candidate.clone();
        state.highest_sequence = 1;
        state.highest_manifest_sha256 = "d".repeat(64);
        state.pending = Some(Pending {
            from: bootstrap.clone(),
            candidate,
            phase: "activated".into(),
        });
        save_state(&dir.0, &state).unwrap();
        let result = recover(&dir.0).unwrap();
        assert_eq!(result["recovered"], true);
        let state = load_state(&dir.0).unwrap();
        assert_eq!(state.active, bootstrap);
        assert_eq!(state.highest_sequence, 1);
        assert!(state.pending.is_none());
    }

    #[test]
    fn corruption_recovers_bootstrap_but_fails_closed_when_all_copies_are_damaged() {
        let dir = TestDir::new();
        let bootstrap = write_slot(&dir.0, true, 0, b"bootstrap data");
        let candidate = write_slot(&dir.0, false, 1, b"candidate data");
        let mut state = initial_state();
        state.bootstrap = bootstrap.clone();
        state.active = candidate.clone();
        state.highest_sequence = 1;
        state.highest_manifest_sha256 = "d".repeat(64);
        save_state(&dir.0, &state).unwrap();
        fs::write(slot_path(&dir.0, &candidate).unwrap(), b"bad").unwrap();
        recover(&dir.0).unwrap();
        assert_eq!(load_state(&dir.0).unwrap().active, bootstrap);
        fs::write(slot_path(&dir.0, &bootstrap).unwrap(), b"bad").unwrap();
        assert!(recover(&dir.0).is_err());
    }

    #[test]
    fn exclusive_update_lock_prevents_concurrent_mutations() {
        let dir = TestDir::new();
        let held = lock(&dir.0).unwrap();
        assert!(lock(&dir.0).is_err());
        drop(held);
        assert!(lock(&dir.0).is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn writable_state_and_symlink_slots_are_rejected() {
        use std::os::unix::{fs::symlink, fs::PermissionsExt};
        let dir = TestDir::new();
        let bootstrap = write_slot(&dir.0, true, 0, b"bootstrap data");
        let mut state = initial_state();
        state.bootstrap = bootstrap.clone();
        state.active = bootstrap.clone();
        save_state(&dir.0, &state).unwrap();
        fs::set_permissions(dir.0.join(STATE_FILE), fs::Permissions::from_mode(0o666)).unwrap();
        assert!(load_state(&dir.0).is_err());
        let path = slot_path(&dir.0, &bootstrap).unwrap();
        fs::remove_file(&path).unwrap();
        symlink("/bin/sh", &path).unwrap();
        assert!(verify_slot(&dir.0, &bootstrap).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn health_checks_identity_exit_status_and_output_limits() {
        let dir = TestDir::new();
        let good = format!("#!/bin/sh\nprintf '%s\\n' '{{\"healthy\":true,\"protocol\":1,\"version\":\"0.2.0\",\"target\":\"{}\"}}'\n",target());
        let slot = write_slot(&dir.0, false, 1, good.as_bytes());
        health(&dir.0, &slot).unwrap();
        for bytes in [
            b"#!/bin/sh\nexit 1\n".as_slice(),
            b"#!/bin/sh\nprintf '%s\\n' '{\"healthy\":true,\"protocol\":1,\"version\":\"bad\"}'\n"
                .as_slice(),
            b"#!/bin/sh\ni=0; while [ $i -lt 20000 ]; do printf x; i=$((i+1)); done\n".as_slice(),
        ] {
            let slot = write_slot(&dir.0, false, 2, bytes);
            assert!(health(&dir.0, &slot).is_err());
        }
    }
}
