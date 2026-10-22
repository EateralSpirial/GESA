//! External configuration and secret storage. Command arguments never contain credentials.
use anyhow::{anyhow, bail, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::net::IpAddr;
use std::path::{Path, PathBuf};
#[cfg(any(target_os = "linux", windows))]
use std::process::{Command, Stdio};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use url::Url;
use zeroize::Zeroize;

const MAX_CONFIG_BYTES: u64 = 32 * 1024;
const MAX_SECRET_BYTES: u64 = 64 * 1024;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub schema: u32,
    pub center_url: String,
    pub account: String,
    pub release_public_key: String,
    #[serde(default = "default_channel")]
    pub channel: String,
    #[serde(default)]
    pub allow_http_loopback: bool,
    #[serde(default = "default_store")]
    pub credential_store: String,
}

fn default_channel() -> String {
    "stable".into()
}
fn default_store() -> String {
    "os".into()
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Credentials {
    pub password: String,
    pub passphrase: String,
}

impl Drop for Credentials {
    fn drop(&mut self) {
        self.password.zeroize();
        self.passphrase.zeroize();
    }
}

impl Config {
    pub fn load(home: &Path) -> Result<Self> {
        ensure_private_dir(home)?;
        let bytes = read_private_file(&home.join("config.json"), MAX_CONFIG_BYTES)
            .context("Configuration unavailable; run gesa init")?;
        let config: Self = serde_json::from_slice(&bytes).context("Invalid configuration JSON")?;
        config.validate()?;
        Ok(config)
    }

    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema == 1, "Unsupported configuration schema");
        ensure!(
            !self.account.trim().is_empty() && self.account.len() <= 256,
            "Account must contain 1–256 bytes"
        );
        ensure!(
            !self.account.chars().any(char::is_control),
            "Account contains control characters"
        );
        let key =
            hex::decode(&self.release_public_key).context("Signing key must be hexadecimal")?;
        ensure!(key.len() == 32, "Signing key must contain 32 bytes");
        ensure!(
            !self.channel.is_empty()
                && self.channel.len() <= 32
                && self
                    .channel
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || c == b'-'),
            "Invalid release channel"
        );
        ensure!(
            matches!(self.credential_store.as_str(), "os" | "file"),
            "Credential store must be os or file"
        );
        #[cfg(not(unix))]
        ensure!(
            self.credential_store == "os",
            "File credentials are available only on Unix development hosts"
        );
        let url = Url::parse(&self.center_url).context("Invalid center URL")?;
        ensure!(
            url.host_str().is_some() && url.username().is_empty() && url.password().is_none(),
            "Center URL must have a host and must omit embedded credentials"
        );
        ensure!(
            url.query().is_none() && url.fragment().is_none(),
            "Center URL must omit query and fragment"
        );
        ensure!(
            url.path() == "/" || url.path().is_empty(),
            "Center URL must be an origin without a path prefix"
        );
        if url.scheme() == "https" {
            return Ok(());
        }
        let numeric_loopback = match url.host() {
            Some(url::Host::Ipv4(ip)) => IpAddr::V4(ip).is_loopback(),
            Some(url::Host::Ipv6(ip)) => IpAddr::V6(ip).is_loopback(),
            _ => false,
        };
        ensure!(
            url.scheme() == "http" && self.allow_http_loopback && numeric_loopback,
            "HTTPS is required; explicit development HTTP permits numeric loopback only"
        );
        Ok(())
    }

    pub fn client(&self) -> Result<reqwest::blocking::Client> {
        self.validate()?;
        reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .redirect(reqwest::redirect::Policy::none())
            .user_agent("gesa-prototype/0.1")
            .build()
            .map_err(|_| anyhow!("Unable to initialize HTTPS client"))
    }

    pub fn endpoint(&self, path: &str) -> Result<Url> {
        self.validate()?;
        ensure!(
            path.starts_with('/') && !path.starts_with("//") && !path.contains('\\'),
            "Endpoint must be an absolute path on the configured center"
        );
        let base = Url::parse(&self.center_url)?;
        let endpoint = base.join(path)?;
        ensure!(
            endpoint.origin() == base.origin(),
            "Endpoint escaped configured center"
        );
        Ok(endpoint)
    }

    pub(crate) fn binding(&self) -> Result<String> {
        let url = Url::parse(&self.center_url)?;
        // Include the entire trust context; editing the pin also invalidates a session.
        Ok(hex::encode(Sha256::digest(format!(
            "{}\n{}\n{}\n{}",
            url.origin().ascii_serialization(),
            self.account,
            self.release_public_key,
            self.channel
        ))))
    }
}

pub fn home_dir() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("GESA_HOME") {
        ensure!(!path.is_empty(), "GESA_HOME is empty");
        let path = PathBuf::from(path);
        ensure!(path.is_absolute(), "GESA_HOME must be an absolute path");
        return Ok(path);
    }
    #[cfg(windows)]
    {
        Ok(
            PathBuf::from(std::env::var_os("LOCALAPPDATA").context("LOCALAPPDATA is unavailable")?)
                .join("GESA"),
        )
    }
    #[cfg(not(windows))]
    {
        if let Some(path) = std::env::var_os("XDG_CONFIG_HOME") {
            let path = PathBuf::from(path);
            ensure!(path.is_absolute(), "XDG_CONFIG_HOME must be absolute");
            return Ok(path.join("gesa"));
        }
        Ok(
            PathBuf::from(std::env::var_os("HOME").context("HOME is unavailable")?)
                .join(".config/gesa"),
        )
    }
}

pub fn initialize(home: &Path) -> Result<()> {
    ensure!(
        !home.join("config.json").exists(),
        "Configuration already exists; edit its non-secret fields directly"
    );
    ensure_private_dir(home)?;
    let center_url = prompt("Center HTTPS origin: ")?;
    let account = prompt("Account: ")?;
    let release_public_key = prompt("Trusted release public key (64 hexadecimal characters): ")?;
    let allow_http_loopback = if center_url.starts_with("http:") {
        prompt("Enable numeric loopback HTTP for local development? [y/N]: ")? == "y"
    } else {
        false
    };
    let selected_store =
        prompt("Credential store [os/file; default os, file is development-only]: ")?;
    let config = Config {
        schema: 1,
        center_url,
        account,
        release_public_key,
        channel: default_channel(),
        allow_http_loopback,
        credential_store: if selected_store.is_empty() {
            default_store()
        } else {
            selected_store
        },
    };
    config.validate()?;
    configure_credentials(home, &config)?;
    write_private_file(
        &home.join("config.json"),
        &serde_json::to_vec_pretty(&config)?,
    )?;
    Ok(())
}

/// Replace credentials for the current external configuration, invalidating its cached session.
pub fn configure_credentials(home: &Path, config: &Config) -> Result<()> {
    config.validate()?;
    ensure_private_dir(home)?;
    let credentials = Credentials {
        password: rpassword::prompt_password("Password: ")
            .context("Unable to read hidden password")?,
        passphrase: rpassword::prompt_password("Passphrase: ")
            .context("Unable to read hidden passphrase")?,
    };
    store_credentials(home, config, &credentials)?;
    delete_secret(home, config, "session")
}

fn prompt(label: &str) -> Result<String> {
    eprint!("{label}");
    io::stderr().flush()?;
    let mut input = String::new();
    ensure!(io::stdin().read_line(&mut input)? > 0, "Input ended");
    Ok(input.trim().to_owned())
}

pub fn store_credentials(home: &Path, config: &Config, credentials: &Credentials) -> Result<()> {
    config.validate()?;
    ensure!(
        !credentials.password.is_empty() && !credentials.passphrase.is_empty(),
        "Password and passphrase are both required"
    );
    ensure!(
        credentials.password.len() <= 4096 && credentials.passphrase.len() <= 4096,
        "Credential exceeds maximum length"
    );
    let mut bytes = serde_json::to_vec(credentials)?;
    let result = store_secret(home, config, "credentials", &bytes);
    bytes.zeroize();
    result
}

pub fn load_credentials(home: &Path, config: &Config) -> Result<Credentials> {
    let mut bytes = read_secret(home, config, "credentials")?
        .context("Credentials unavailable; configure the selected credential store")?;
    let result = serde_json::from_slice(&bytes).map_err(|_| anyhow!("Invalid credential record"));
    bytes.zeroize();
    result
}

fn secret_id(home: &Path, config: &Config, purpose: &str) -> Result<String> {
    ensure!(
        matches!(purpose, "credentials" | "session"),
        "Invalid secret purpose"
    );
    let home = fs::canonicalize(home).context("GESA home unavailable")?;
    Ok(hex::encode(Sha256::digest(format!(
        "{}\n{}\n{}",
        home.display(),
        config.binding()?,
        purpose
    ))))
}

fn secret_path(home: &Path, config: &Config, purpose: &str) -> Result<PathBuf> {
    Ok(home.join(format!(
        "{purpose}-{}.secret",
        secret_id(home, config, purpose)?
    )))
}

pub(crate) fn store_secret(
    home: &Path,
    config: &Config,
    purpose: &str,
    bytes: &[u8],
) -> Result<()> {
    ensure_private_dir(home)?;
    ensure!(
        bytes.len() as u64 <= MAX_SECRET_BYTES,
        "Secret record too large"
    );
    if config.credential_store == "file" {
        #[cfg(unix)]
        {
            return write_private_file(&secret_path(home, config, purpose)?, bytes);
        }
        #[cfg(not(unix))]
        {
            bail!("Plaintext secret files are unavailable on this platform");
        }
    }
    #[cfg(target_os = "linux")]
    {
        let mut command = Command::new("secret-tool");
        command.args([
            "store",
            "--label=GESA CLI",
            "application",
            "gesa",
            "record",
            &secret_id(home, config, purpose)?,
        ]);
        let output = bounded_process(command, bytes)?;
        ensure!(
            output.status.success(),
            "OS vault could not store the secret"
        );
        Ok(())
    }
    #[cfg(windows)]
    {
        let encrypted = windows_protect(bytes, true)?;
        write_private_file(&secret_path(home, config, purpose)?, &encrypted)
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        bail!(
            "OS vault adapter is available on Linux and Windows; Unix development supports explicit file mode"
        );
    }
}

pub(crate) fn read_secret(home: &Path, config: &Config, purpose: &str) -> Result<Option<Vec<u8>>> {
    ensure_private_dir(home)?;
    if config.credential_store == "file" {
        #[cfg(unix)]
        {
            let path = secret_path(home, config, purpose)?;
            return match fs::symlink_metadata(&path) {
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
                Err(_) => Err(anyhow!("Secret record unavailable")),
                Ok(_) => Ok(Some(read_private_file(&path, MAX_SECRET_BYTES)?)),
            };
        }
        #[cfg(not(unix))]
        {
            bail!("Plaintext secret files are unavailable on this platform");
        }
    }
    #[cfg(target_os = "linux")]
    {
        let mut command = Command::new("secret-tool");
        command.args([
            "lookup",
            "application",
            "gesa",
            "record",
            &secret_id(home, config, purpose)?,
        ]);
        let output = bounded_process(command, &[])?;
        if !output.status.success() {
            ensure!(output.status.code() == Some(1), "OS vault lookup failed");
            return Ok(None);
        }
        ensure!(
            output.stdout.len() as u64 <= MAX_SECRET_BYTES,
            "OS vault record too large"
        );
        Ok(Some(output.stdout))
    }
    #[cfg(windows)]
    {
        let path = secret_path(home, config, purpose)?;
        if !path.try_exists()? {
            return Ok(None);
        }
        Ok(Some(windows_protect(
            &read_private_file(&path, MAX_SECRET_BYTES)?,
            false,
        )?))
    }
    #[cfg(not(any(target_os = "linux", windows)))]
    {
        bail!("OS vault adapter is available on Linux and Windows");
    }
}

pub(crate) fn delete_secret(home: &Path, config: &Config, purpose: &str) -> Result<()> {
    if config.credential_store == "file" || cfg!(windows) {
        let path = secret_path(home, config, purpose)?;
        match fs::remove_file(path) {
            Ok(()) => return Ok(()),
            Err(e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(_) => bail!("Secret record could not be removed"),
        }
    }
    #[cfg(target_os = "linux")]
    {
        let mut command = Command::new("secret-tool");
        command.args([
            "clear",
            "application",
            "gesa",
            "record",
            &secret_id(home, config, purpose)?,
        ]);
        let status = bounded_process(command, &[])?.status;
        ensure!(
            status.success() || status.code() == Some(1),
            "OS vault delete failed"
        );
        Ok(())
    }
    #[cfg(not(target_os = "linux"))]
    {
        bail!("OS vault adapter unavailable");
    }
}

pub fn ensure_private_dir(home: &Path) -> Result<()> {
    if !home.try_exists()? {
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            fs::DirBuilder::new()
                .recursive(true)
                .mode(0o700)
                .create(home)?;
        }
        #[cfg(not(unix))]
        fs::create_dir_all(home)?;
        #[cfg(windows)]
        windows_harden_path(home, true)?;
    }
    let metadata = fs::symlink_metadata(home)?;
    ensure!(
        metadata.is_dir() && !metadata.file_type().is_symlink(),
        "GESA home must be a real directory"
    );
    check_private(&metadata, true)?;
    #[cfg(windows)]
    windows_check_acl(home, &metadata)?;
    Ok(())
}

/// Harden a regular executable file newly created by the caller inside a private directory.
/// Existing installed files should be verified, rather than silently changing their permissions.
pub fn harden_new_executable(path: &Path) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "New executable must be a regular file"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        ensure!(
            metadata.file_attributes() & 0x400 == 0,
            "New executable must omit Windows reparse points"
        );
        windows_harden_path(path, false)?;
        windows_check_acl(path, &fs::symlink_metadata(path)?)?;
    }
    Ok(())
}

fn check_private(metadata: &fs::Metadata, directory: bool) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        unsafe extern "C" {
            fn getuid() -> u32;
        }
        let uid = unsafe { getuid() };
        ensure!(
            metadata.uid() == uid,
            "GESA files must be owned by the current user"
        );
        ensure!(
            metadata.mode() & 0o077 == 0,
            "GESA files must be private (directory 0700, files 0600)"
        );
        if !directory {
            ensure!(
                metadata.mode() & 0o111 == 0,
                "Secret/config files must omit executable permissions"
            );
        }
    }
    #[cfg(not(unix))]
    let _ = (metadata, directory);
    Ok(())
}

pub fn read_private_file(path: &Path, limit: u64) -> Result<Vec<u8>> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "Expected a regular private file"
    );
    check_private(&metadata, false)?;
    #[cfg(windows)]
    windows_check_acl(path, &metadata)?;
    ensure!(metadata.len() <= limit, "Private file exceeds size limit");
    let file = File::open(path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        let opened = file.metadata()?;
        ensure!(
            opened.ino() == metadata.ino() && opened.dev() == metadata.dev(),
            "Private file changed during read"
        );
        check_private(&opened, false)?;
    }
    let mut bytes = Vec::new();
    file.take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "Private file exceeds size limit"
    );
    Ok(bytes)
}

pub fn write_private_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let parent = path
        .parent()
        .context("Private file needs a parent directory")?;
    ensure_private_dir(parent)?;
    if let Ok(meta) = fs::symlink_metadata(path) {
        ensure!(
            meta.is_file() && !meta.file_type().is_symlink(),
            "Refusing to replace a non-regular file"
        );
        check_private(&meta, false)?;
        #[cfg(windows)]
        windows_check_acl(path, &meta)?;
    }
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let temp = parent.join(format!(".write-{}-{stamp}", std::process::id()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(&temp)?;
    #[cfg(windows)]
    windows_harden_path(&temp, false)?;
    let result = (|| -> Result<()> {
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::rename(&temp, path)?;
        #[cfg(unix)]
        File::open(parent)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

/// Helpers receive sensitive values on stdin. A locked keyring cannot stall an update indefinitely.
#[cfg(any(target_os = "linux", windows))]
fn bounded_process(mut command: Command, input: &[u8]) -> Result<std::process::Output> {
    let mut child = command
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| {
            anyhow!(
                "OS helper unavailable; Linux requires secret-tool and an unlocked desktop keyring"
            )
        })?;
    let mut stdin = child.stdin.take().context("OS helper input unavailable")?;
    let stdout = child
        .stdout
        .take()
        .context("OS helper output unavailable")?;
    let mut input = input.to_vec();
    let writer = std::thread::spawn(move || {
        let result = stdin.write_all(&input);
        input.zeroize();
        result
    });
    let reader = std::thread::spawn(move || {
        let mut bytes = Vec::new();
        stdout.take(MAX_SECRET_BYTES + 1).read_to_end(&mut bytes)?;
        Ok::<_, io::Error>(bytes)
    });
    let deadline = std::time::Instant::now() + Duration::from_secs(20);
    let status = loop {
        if let Some(status) = child.try_wait()? {
            break Some(status);
        }
        if std::time::Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        std::thread::sleep(Duration::from_millis(25));
    };
    let written = writer
        .join()
        .map_err(|_| anyhow!("OS helper input failed"))?;
    let read = reader
        .join()
        .map_err(|_| anyhow!("OS helper output failed"))?;
    let status = status.context("OS helper timed out; unlock the keyring before retrying")?;
    written.map_err(|_| anyhow!("OS helper input failed"))?;
    let stdout = read.map_err(|_| anyhow!("OS helper output failed"))?;
    ensure!(
        stdout.len() as u64 <= MAX_SECRET_BYTES,
        "OS helper output exceeded limit"
    );
    Ok(std::process::Output {
        status,
        stdout,
        stderr: Vec::new(),
    })
}

/// Validate Windows ownership and writable ACLs before trusting configuration or update state.
/// PowerShell receives only the path through stdin; credential material never reaches its arguments.
#[cfg(windows)]
fn windows_check_acl(path: &Path, metadata: &fs::Metadata) -> Result<()> {
    use std::os::windows::fs::MetadataExt;
    ensure!(
        metadata.file_attributes() & 0x400 == 0,
        "GESA paths must omit Windows reparse points"
    );
    let path = path.to_str().context("GESA path must be valid Unicode")?;
    ensure!(
        !path.contains(['\r', '\n']),
        "GESA path contains a line break"
    );
    let script = r#"
$ErrorActionPreference='Stop'
try {
  [Console]::InputEncoding=[System.Text.UTF8Encoding]::new($false)
  $path=[Console]::In.ReadLine()
  $acl=Get-Acl -LiteralPath $path
  $sidType=[System.Security.Principal.SecurityIdentifier]
  $current=[System.Security.Principal.WindowsIdentity]::GetCurrent().User.Value
  if($acl.GetOwner($sidType).Value -ne $current) { exit 3 }
  $raw=[System.Security.AccessControl.RawSecurityDescriptor]::new($acl.GetSecurityDescriptorBinaryForm(),0)
  if($null -eq $raw.DiscretionaryAcl) { exit 3 }
  $allowed=@($current,'S-1-5-18','S-1-5-32-544','S-1-3-0')
  foreach($rule in $acl.Access) {
    if($rule.AccessControlType -eq 'Allow' -and (([long]$rule.FileSystemRights -band 0x500D0156) -ne 0)) {
      if($allowed -notcontains $rule.IdentityReference.Translate($sidType).Value) { exit 3 }
    }
  }
  exit 0
} catch { exit 3 }
"#;
    let system_root =
        std::env::var_os("SystemRoot").context("Windows system directory unavailable")?;
    let mut command = Command::new(
        PathBuf::from(system_root).join("System32/WindowsPowerShell/v1.0/powershell.exe"),
    );
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        script,
    ]);
    let output = bounded_process(command, format!("{path}\n").as_bytes())?;
    ensure!(output.status.success(), "GESA files require current-user ownership and write access limited to the user, SYSTEM and Administrators");
    Ok(())
}

/// Set a private DACL only on a path created by this invocation; never rewrite an existing ACL.
#[cfg(windows)]
fn windows_harden_path(path: &Path, directory: bool) -> Result<()> {
    let path = path.to_str().context("GESA path must be valid Unicode")?;
    ensure!(
        !path.contains('\r') && !path.contains('\n'),
        "GESA path contains a line break"
    );
    let script = r#"
$ErrorActionPreference='Stop'
try {
  [Console]::InputEncoding=[System.Text.UTF8Encoding]::new($false)
  $path=[Console]::In.ReadLine()
  $isDirectory=([Console]::In.ReadLine() -eq 'directory')
  $current=[System.Security.Principal.WindowsIdentity]::GetCurrent().User
  if($isDirectory) {
    $acl=[System.Security.AccessControl.DirectorySecurity]::new()
    $inherit=[System.Security.AccessControl.InheritanceFlags]'ContainerInherit,ObjectInherit'
  } else {
    $acl=[System.Security.AccessControl.FileSecurity]::new()
    $inherit=[System.Security.AccessControl.InheritanceFlags]::None
  }
  $acl.SetOwner($current)
  $acl.SetAccessRuleProtection($true,$false)
  foreach($sid in @($current.Value,'S-1-5-18','S-1-5-32-544')) {
    $identity=[System.Security.Principal.SecurityIdentifier]::new($sid)
    $rule=[System.Security.AccessControl.FileSystemAccessRule]::new($identity,'FullControl',$inherit,'None','Allow')
    $acl.AddAccessRule($rule)
  }
  Set-Acl -LiteralPath $path -AclObject $acl
  exit 0
} catch { exit 3 }
"#;
    let system_root =
        std::env::var_os("SystemRoot").context("Windows system directory unavailable")?;
    let mut command = Command::new(
        PathBuf::from(system_root).join("System32/WindowsPowerShell/v1.0/powershell.exe"),
    );
    command.args([
        "-NoLogo",
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        script,
    ]);
    let kind = if directory { "directory" } else { "file" };
    let output = bounded_process(command, format!("{path}\n{kind}\n").as_bytes())?;
    ensure!(
        output.status.success(),
        "Unable to set a private Windows ACL"
    );
    Ok(())
}

#[cfg(windows)]
fn windows_protect(bytes: &[u8], protect: bool) -> Result<Vec<u8>> {
    use std::ffi::c_void;
    #[repr(C)]
    struct Blob {
        len: u32,
        data: *mut u8,
    }
    #[link(name = "crypt32")]
    unsafe extern "system" {
        fn CryptProtectData(
            input: *const Blob,
            description: *const u16,
            entropy: *const Blob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut Blob,
        ) -> i32;
        fn CryptUnprotectData(
            input: *const Blob,
            description: *mut *mut u16,
            entropy: *const Blob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            output: *mut Blob,
        ) -> i32;
    }
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn LocalFree(memory: *mut c_void) -> *mut c_void;
    }
    ensure!(bytes.len() <= u32::MAX as usize, "Secret too large");
    let input = Blob {
        len: bytes.len() as u32,
        data: bytes.as_ptr() as *mut u8,
    };
    let mut output = Blob {
        len: 0,
        data: std::ptr::null_mut(),
    };
    let result = unsafe {
        if protect {
            CryptProtectData(
                &input,
                std::ptr::null(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                1,
                &mut output,
            )
        } else {
            CryptUnprotectData(
                &input,
                std::ptr::null_mut(),
                std::ptr::null(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                1,
                &mut output,
            )
        }
    };
    ensure!(result != 0, "Windows credential protection failed");
    let value = unsafe { std::slice::from_raw_parts(output.data, output.len as usize).to_vec() };
    unsafe {
        std::ptr::write_bytes(output.data, 0, output.len as usize);
        LocalFree(output.data.cast());
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(crate) fn example() -> Config {
        Config {
            schema: 1,
            center_url: "https://center.example".into(),
            account: "tester".into(),
            release_public_key: "11".repeat(32),
            channel: "stable".into(),
            allow_http_loopback: false,
            credential_store: "file".into(),
        }
    }

    #[test]
    fn validates_transport_and_endpoint_origin() {
        let mut c = example();
        c.credential_store = "os".into();
        assert!(c.validate().is_ok());
        assert!(c.endpoint("https://evil.example").is_err());
        assert!(c.endpoint("//evil.example/path").is_err());
        assert!(c.endpoint("/\\evil.example/path").is_err());
        for url in [
            "http://example.com",
            "http://127.0.0.1",
            "file:///tmp/center",
            "https://u:p@example.com",
            "https://example.com/path",
            "https://example.com/?token=secret",
        ] {
            c.center_url = url.into();
            assert!(c.validate().is_err(), "accepted {url}");
        }
        c.allow_http_loopback = true;
        for url in ["http://127.0.0.1:8000", "http://[::1]:8000"] {
            c.center_url = url.into();
            assert!(c.validate().is_ok());
        }
        for url in [
            "http://localhost:8000",
            "http://127.0.0.1.example.com",
            "http://0.0.0.0:8000",
        ] {
            c.center_url = url.into();
            assert!(c.validate().is_err());
        }
    }

    #[test]
    fn rejects_secrets_and_unknown_config_fields() {
        let c = example();
        let mut json = serde_json::to_value(c).unwrap();
        json["password"] = "secret".into();
        assert!(serde_json::from_value::<Config>(json).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn private_credentials_roundtrip_and_permission_checks() {
        use std::os::unix::fs::{symlink, PermissionsExt};
        let home = std::env::temp_dir().join(format!(
            "gesa-credentials-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let c = example();
        ensure_private_dir(&home).unwrap();
        let creds = Credentials {
            password: "pw\nwith spaces".into(),
            passphrase: "phrase".into(),
        };
        store_credentials(&home, &c, &creds).unwrap();
        assert_eq!(
            load_credentials(&home, &c).unwrap().password,
            creds.password
        );
        let path = secret_path(&home, &c, "credentials").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(load_credentials(&home, &c).is_err());
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        let other = home.join("target");
        fs::rename(&path, &other).unwrap();
        symlink(&other, &path).unwrap();
        assert!(load_credentials(&home, &c).is_err());
        assert!(harden_new_executable(&path).is_err());
        assert_eq!(
            fs::metadata(&other).unwrap().permissions().mode() & 0o777,
            0o600
        );
        fs::remove_dir_all(home).unwrap();
    }

    #[test]
    fn identity_binding_changes_with_server_account_or_pin() {
        let c = example();
        let binding = c.binding().unwrap();
        let mut changed = c.clone();
        changed.center_url = "https://other.example".into();
        assert_ne!(binding, changed.binding().unwrap());
        changed = c.clone();
        changed.account = "another".into();
        assert_ne!(binding, changed.binding().unwrap());
        changed = c;
        changed.release_public_key = "22".repeat(32);
        assert_ne!(binding, changed.binding().unwrap());
    }

    #[cfg(windows)]
    #[test]
    fn windows_private_acl_and_dpapi_roundtrip() {
        let home = std::env::temp_dir().join(format!(
            "gesa-凭据-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        ensure_private_dir(&home).unwrap();
        let mut c = example();
        c.credential_store = "os".into();
        let credentials = Credentials {
            password: "测试 password".into(),
            passphrase: "another phrase".into(),
        };
        store_credentials(&home, &c, &credentials).unwrap();
        assert_eq!(
            load_credentials(&home, &c).unwrap().password,
            credentials.password
        );
        write_private_file(&home.join("config.json"), &serde_json::to_vec(&c).unwrap()).unwrap();
        assert_eq!(Config::load(&home).unwrap().account, c.account);
        let executable = home.join("core.exe");
        fs::write(&executable, b"MZ-test-fixture").unwrap();
        harden_new_executable(&executable).unwrap();
        let encrypted = fs::read(secret_path(&home, &c, "credentials").unwrap()).unwrap();
        assert!(!encrypted
            .windows(credentials.password.len())
            .any(|bytes| bytes == credentials.password.as_bytes()));
        fs::remove_dir_all(home).unwrap();
    }
}
