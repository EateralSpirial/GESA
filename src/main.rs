use anyhow::{ensure, Context, Result};
use clap::{Parser, Subcommand};
use gesa_cli::{auth, config, protocol, update};
use serde_json::{json, Value};
use std::{path::PathBuf, time::Duration};

#[derive(Parser)]
#[command(name="gesa", version=gesa_cli::BUILD_VERSION, about="GESA verifiable update protocol prototype")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Configure the central service and vault through interactive prompts.
    Init,
    /// Install the independent launcher from the matching distribution directory.
    Install,
    /// Authenticate with the credentials stored outside the command line.
    Login,
    /// Replace saved password and passphrase through hidden prompts.
    Credentials,
    /// Remove the locally cached session.
    Logout,
    /// Show installed versions and update state, without secrets.
    Status,
    /// Check signed release metadata and deterministic iteration gates.
    Check,
    /// Stage, verify, health-check and activate the latest release.
    Update,
    /// Restore the previous verified local executable.
    Rollback,
    /// Recover an interrupted activation using retained local binaries.
    Recover,
    /// Recompute hashes of retained executables and validate local state.
    Verify,
    /// Poll the center; transient failures leave the active version available.
    Watch {
        #[arg(long, default_value_t=60, value_parser=clap::value_parser!(u64).range(5..=86400))]
        interval: u64,
        #[arg(long)]
        once: bool,
    },
    /// Validate an iteration evidence file without downloading or executing code.
    Protocol {
        file: PathBuf,
        #[arg(long, default_value_t = 1)]
        sequence: u64,
        #[arg(long, default_value_t = 1)]
        epoch: u32,
        #[arg(long, value_delimiter = ',', default_value = "1")]
        supports: Vec<u32>,
    },
    #[command(name = "__health", hide = true)]
    Health,
}

fn emit(command: &str, result: Value) {
    println!(
        "{}",
        json!({"schema":1,"ok":true,"version":gesa_cli::BUILD_VERSION,"command":command,"result":result})
    );
}

fn run(cli: Cli) -> Result<()> {
    if matches!(cli.command, Command::Health) {
        let version = gesa_cli::BUILD_VERSION;
        println!(
            "{}",
            json!({"healthy":true,"protocol":1,"version":version,"target":format!("{}-{}",std::env::consts::OS,std::env::consts::ARCH)})
        );
        return Ok(());
    }
    if let Command::Protocol {
        file,
        sequence,
        epoch,
        supports,
    } = cli.command
    {
        ensure!(
            std::fs::metadata(&file)?.len() <= 256 * 1024,
            "iteration file too large"
        );
        let value =
            serde_json::from_slice(&std::fs::read(file)?).context("invalid iteration JSON")?;
        protocol::validate(&value, sequence, epoch, &supports)?;
        emit(
            "protocol",
            json!({"valid":true,"sequence":sequence,"contract_epoch":epoch}),
        );
        return Ok(());
    }
    let home = config::home_dir()?;
    match cli.command {
        Command::Init => {
            config::initialize(&home)?;
            update::install_bootstrap(&home)?;
            emit("init", json!({"configured":true,"home":home}));
        }
        Command::Install => {
            update::install_bootstrap(&home)?;
            let source = std::env::current_exe()?
                .with_file_name(format!("gesa-launcher{}", std::env::consts::EXE_SUFFIX));
            ensure!(source.is_file(), "Place gesa and gesa-launcher from the same distribution together, then run gesa install");
            let bin = home.join("bin");
            config::ensure_private_dir(&bin)?;
            let target = bin.join(format!("gesa{}", std::env::consts::EXE_SUFFIX));
            if let Ok(meta) = std::fs::symlink_metadata(&target) {
                ensure!(
                    meta.is_file() && !meta.file_type().is_symlink(),
                    "Launcher destination must be a regular file"
                );
            }
            let temporary = bin.join(format!(".launcher-{}.tmp", rand::random::<u64>()));
            let mut input = std::fs::File::open(source)?;
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o700);
            }
            let installed = (|| -> Result<()> {
                let mut output = options.open(&temporary)?;
                config::harden_new_executable(&temporary)?;
                std::io::copy(&mut input, &mut output)?;
                output.sync_all()?;
                drop(output);
                std::fs::rename(&temporary, &target)?;
                #[cfg(unix)]
                std::fs::File::open(&bin)?.sync_all()?;
                Ok(())
            })();
            if installed.is_err() {
                let _ = std::fs::remove_file(&temporary);
            }
            installed?;
            emit("install", json!({"launcher":target,"path_directory":bin}));
        }
        Command::Login => {
            let c = config::Config::load(&home)?;
            let session = auth::login(&home, &c)?;
            emit(
                "login",
                json!({"authenticated":true,"expires_at":session.expires_at}),
            );
        }
        Command::Credentials => {
            let c = config::Config::load(&home)?;
            config::configure_credentials(&home, &c)?;
            emit("credentials", json!({"saved":true}));
        }
        Command::Logout => {
            let c = config::Config::load(&home)?;
            auth::logout(&home, &c)?;
            emit("logout", json!({"local_session_removed":true}));
        }
        Command::Status => emit("status", update::status(&home)?),
        Command::Verify => emit("verify", update::verify(&home)?),
        Command::Recover => emit("recover", update::recover(&home)?),
        Command::Rollback => emit("rollback", update::rollback(&home)?),
        Command::Check => emit(
            "check",
            update::check(&home, &config::Config::load(&home)?)?,
        ),
        Command::Update => emit(
            "update",
            update::apply(&home, &config::Config::load(&home)?)?,
        ),
        Command::Watch { interval, once } => {
            loop {
                // Reload configuration each cycle to pick up operator changes.
                let result = config::Config::load(&home).and_then(|c| update::apply(&home, &c));
                match result {
                    Ok(v) => emit("watch", v),
                    Err(e) if once => return Err(e),
                    Err(_) => eprintln!(
                        "{}",
                        json!({"schema":1,"ok":false,"command":"watch","error":{"code":"UPDATE_RETRY","message":"Update attempt failed; retained version remains available. Retry on next interval."}})
                    ),
                }
                if once {
                    break;
                }
                std::thread::sleep(Duration::from_secs(interval));
            }
        }
        Command::Protocol { .. } | Command::Health => unreachable!(),
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();
    if let Err(error) = run(cli) {
        eprintln!(
            "{}",
            json!({"schema":1,"ok":false,"error":{"code":"GESA_FAILED","message":error.to_string()}})
        );
        std::process::exit(1);
    }
}
