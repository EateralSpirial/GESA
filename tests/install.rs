//! Exercise the real executable/launcher installation path on every CI platform.
//! No center, network request, credential store, or interactive input is needed.
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output, Stdio},
};

struct Installation(PathBuf);

impl Installation {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "gesa-install-test-{}-{}",
            std::process::id(),
            rand::random::<u64>()
        ));
        gesa_cli::config::ensure_private_dir(&root).unwrap();
        Self(root)
    }
}

impl Drop for Installation {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn invoke(executable: &Path, home: &Path, argument: &str) -> Output {
    Command::new(executable)
        .arg(argument)
        .env("GESA_HOME", home)
        .stdin(Stdio::null())
        .output()
        .expect("real GESA executable must start")
}

fn success_json(output: Output) -> Value {
    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("command must return JSON")
}

#[test]
fn installs_twice_and_launcher_forwards_status_health_and_exit_code() {
    let installation = Installation::new();
    let distribution = installation.0.join("distribution");
    gesa_cli::config::ensure_private_dir(&distribution).unwrap();
    let core = distribution.join(format!("gesa{}", std::env::consts::EXE_SUFFIX));
    let packaged_launcher =
        distribution.join(format!("gesa-launcher{}", std::env::consts::EXE_SUFFIX));
    fs::copy(env!("CARGO_BIN_EXE_gesa"), &core).unwrap();
    fs::copy(env!("CARGO_BIN_EXE_gesa-launcher"), &packaged_launcher).unwrap();
    let home = installation.0.join("home");

    let first = success_json(invoke(&core, &home, "install"));
    assert_eq!(first["command"], "install");
    assert_eq!(first["ok"], true);
    assert!(
        !home.join("config.json").exists(),
        "installation must work before credential configuration"
    );
    let initial_state = fs::read(home.join("update-state.json")).unwrap();
    let second = success_json(invoke(&core, &home, "install"));
    assert_eq!(second["ok"], true);
    assert_eq!(
        initial_state,
        fs::read(home.join("update-state.json")).unwrap(),
        "reinstall must preserve bootstrap trust and the update high-watermark"
    );

    let installed_launcher = home
        .join("bin")
        .join(format!("gesa{}", std::env::consts::EXE_SUFFIX));
    let status = success_json(invoke(&installed_launcher, &home, "status"));
    assert_eq!(
        status["result"]["bootstrap"]["version"],
        gesa_cli::BUILD_VERSION
    );
    assert_eq!(status["result"]["bootstrap"]["sequence"], 0);
    assert_eq!(status["result"]["active"], status["result"]["bootstrap"]);
    assert_eq!(status["result"]["highest_sequence"], 0);
    assert_eq!(
        status["result"]["bootstrap"]["sha256"],
        hex::encode(Sha256::digest(fs::read(&core).unwrap()))
    );

    let health = success_json(invoke(&installed_launcher, &home, "__health"));
    assert_eq!(health["healthy"], true);
    assert_eq!(health["protocol"], 1);
    assert_eq!(health["version"], gesa_cli::BUILD_VERSION);
    assert_eq!(
        health["target"],
        format!("{}-{}", std::env::consts::OS, std::env::consts::ARCH)
    );

    let rejected = invoke(&installed_launcher, &home, "invalidcommand");
    assert_eq!(
        rejected.status.code(),
        Some(2),
        "launcher must forward core argument-parser exit status"
    );
    assert!(!rejected.stderr.is_empty());
}
