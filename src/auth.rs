//! Account + password + passphrase checks over the configured TLS origin.
use crate::config::{self, Config};
use anyhow::{anyhow, ensure, Context, Result};
use serde::{Deserialize, Serialize};
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Session {
    pub access_token: String,
    pub expires_at: u64,
}

impl Drop for Session {
    fn drop(&mut self) {
        self.access_token.zeroize();
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredSession {
    binding: String,
    session: Session,
}

fn now() -> Result<u64> {
    Ok(SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs())
}

pub fn login(home: &Path, config: &Config) -> Result<Session> {
    config.validate()?;
    let credentials = config::load_credentials(home, config)?;
    #[derive(Serialize)]
    struct Login<'a> {
        account: &'a str,
        password: &'a str,
        passphrase: &'a str,
    }
    let request = Login {
        account: &config.account,
        password: &credentials.password,
        passphrase: &credentials.passphrase,
    };
    let mut body = serde_json::to_vec(&request)?;
    let response = config
        .client()?
        .post(config.endpoint("/v1/login")?)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(body.clone())
        .send()
        .map_err(|_| anyhow!("Center login request failed"));
    body.zeroize();
    let response = response?;
    ensure!(response.status().is_success(), "Center rejected login");
    // Never include server response text or reqwest error URLs in diagnostics.
    let mut bytes = Vec::new();
    response
        .take(16 * 1024 + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| anyhow!("Invalid center login response"))?;
    ensure!(bytes.len() <= 16 * 1024, "Center login response too large");
    let result: Result<Session> =
        serde_json::from_slice(&bytes).map_err(|_| anyhow!("Invalid center login response"));
    bytes.zeroize();
    let session = result?;
    validate_session(&session, now()?)?;
    let stored = StoredSession {
        binding: config.binding()?,
        session,
    };
    let mut bytes = serde_json::to_vec(&stored)?;
    let result = config::store_secret(home, config, "session", &bytes);
    bytes.zeroize();
    result?;
    Ok(stored.session)
}

fn validate_session(session: &Session, now: u64) -> Result<()> {
    ensure!(
        !session.access_token.is_empty()
            && session.access_token.len() <= 8192
            && session.access_token.bytes().all(|b| b.is_ascii_graphic()),
        "Invalid session token"
    );
    ensure!(
        session.expires_at > now.saturating_add(5),
        "Session has expired"
    );
    ensure!(
        session.expires_at <= now.saturating_add(24 * 60 * 60),
        "Session lifetime exceeds one day"
    );
    Ok(())
}

pub fn token(home: &Path, config: &Config) -> Result<String> {
    config.validate()?;
    if let Some(mut bytes) = config::read_secret(home, config, "session")? {
        let parsed: Result<StoredSession> = serde_json::from_slice(&bytes)
            .map_err(|_| anyhow!("Invalid stored session; run gesa logout"));
        bytes.zeroize();
        let stored = parsed?;
        ensure!(
            stored.binding == config.binding()?,
            "Session belongs to a different trust context"
        );
        if validate_session(&stored.session, now()?).is_ok() {
            return Ok(stored.session.access_token.clone());
        }
    }
    Ok(login(home, config)?.access_token.clone())
}

pub fn logout(home: &Path, config: &Config) -> Result<()> {
    config.validate()?;
    config::delete_secret(home, config, "session").context("Unable to clear local session")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_validation_rejects_expired_long_lived_and_header_injection() {
        let now = 1000;
        for (access_token, expires_at) in [
            ("good", 1000),
            ("good", 1005),
            ("good", 100_000),
            ("bad\r\nheader", 1500),
            ("", 1500),
        ] {
            assert!(validate_session(
                &Session {
                    access_token: access_token.into(),
                    expires_at
                },
                now
            )
            .is_err());
        }
        assert!(validate_session(
            &Session {
                access_token: "abc.def-123".into(),
                expires_at: 1500
            },
            now
        )
        .is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn session_cache_is_bound_and_logout_removes_it() {
        let home = std::env::temp_dir().join(format!(
            "gesa-session-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        config::ensure_private_dir(&home).unwrap();
        let config = Config {
            schema: 1,
            center_url: "https://center.example".into(),
            account: "tester".into(),
            release_public_key: "11".repeat(32),
            channel: "stable".into(),
            allow_http_loopback: false,
            credential_store: "file".into(),
        };
        let mut record = StoredSession {
            binding: config.binding().unwrap(),
            session: Session {
                access_token: "cached-token".into(),
                expires_at: now().unwrap() + 60,
            },
        };
        config::store_secret(
            &home,
            &config,
            "session",
            &serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
        assert_eq!(token(&home, &config).unwrap(), "cached-token");
        record.binding = "unrelated-context".into();
        config::store_secret(
            &home,
            &config,
            "session",
            &serde_json::to_vec(&record).unwrap(),
        )
        .unwrap();
        assert!(token(&home, &config).is_err());
        logout(&home, &config).unwrap();
        assert!(config::read_secret(&home, &config, "session")
            .unwrap()
            .is_none());
        std::fs::remove_dir_all(home).unwrap();
    }
}
