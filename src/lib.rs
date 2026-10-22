pub mod auth;
pub mod config;
pub mod protocol;
pub mod update;

pub const BUILD_VERSION: &str = match option_env!("GESA_RELEASE_VERSION") {
    Some(version) => version,
    None => env!("CARGO_PKG_VERSION"),
};
