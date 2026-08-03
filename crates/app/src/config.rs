//! Application configuration, read from environment variables.
//!
//! The reference server listens on port 3100 by default, so Staple does the
//! same. `HOST` and `PORT` override the bind address, and `RUST_LOG` controls
//! the log level (a `tracing_subscriber::EnvFilter` directive).

use std::{env, error::Error, fmt};

const DEFAULT_HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3100;
const DEFAULT_LOG_FILTER: &str = "info";

/// Staple application configuration.
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Bind host.
    pub host: String,
    /// Bind port.
    pub port: u16,
    /// `EnvFilter` directive for the log level.
    pub log_filter: String,
}

impl AppConfig {
    /// Loads configuration from environment variables, applying defaults.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::InvalidPort`] when `PORT` is not a valid port
    /// number.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::load(|key| env::var(key))
    }

    fn load(
        mut get: impl FnMut(&str) -> Result<String, env::VarError>,
    ) -> Result<Self, ConfigError> {
        let host = match get("HOST") {
            Ok(value) => value,
            Err(_) => DEFAULT_HOST.to_owned(),
        };
        let port = match get("PORT") {
            Ok(value) => value
                .parse::<u16>()
                .map_err(|source| ConfigError::InvalidPort { value, source })?,
            Err(_) => DEFAULT_PORT,
        };
        let log_filter = match get("RUST_LOG") {
            Ok(value) => value,
            Err(_) => DEFAULT_LOG_FILTER.to_owned(),
        };
        Ok(Self {
            host,
            port,
            log_filter,
        })
    }
}

/// Configuration errors.
#[derive(Debug)]
pub enum ConfigError {
    /// `PORT` was set but is not a valid port number.
    InvalidPort {
        /// The offending value.
        value: String,
        /// The parse error.
        source: std::num::ParseIntError,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPort { value, .. } => write!(f, "invalid PORT value: {value:?}"),
        }
    }
}

impl Error for ConfigError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::InvalidPort { source, .. } => Some(source),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn load_with(vars: &[(&str, &str)]) -> Result<AppConfig, ConfigError> {
        let map = vars
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect::<HashMap<_, _>>();
        AppConfig::load(|key| map.get(key).cloned().ok_or(env::VarError::NotPresent))
    }

    #[test]
    fn defaults_apply_when_env_unset() {
        let config = load_with(&[]).unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 3100);
        assert_eq!(config.log_filter, "info");
    }

    #[test]
    fn env_overrides_apply() {
        let config =
            load_with(&[("HOST", "0.0.0.0"), ("PORT", "8080"), ("RUST_LOG", "debug")]).unwrap();
        assert_eq!(config.host, "0.0.0.0");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_filter, "debug");
    }

    #[test]
    fn invalid_port_is_rejected() {
        let error = load_with(&[("PORT", "not-a-port")]).unwrap_err();
        assert!(matches!(error, ConfigError::InvalidPort { .. }));
    }
}
