//! `config.toml`: where the server listens, how long a query may be, and extra exceptions.
//!
//! Every key is optional; a missing file is the same as an empty one. The template with the
//! comments is `config.example.toml`.

use crate::utils::Result;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct Config {
    pub server: ServerConfig,
    pub convert: ConvertConfig,
    /// Word (as UniDic splits it) to the romaji it is replaced by.
    pub exceptions: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        // Only this machine, so that it has to be opened up on purpose (behind a reverse proxy).
        Self { host: "127.0.0.1".to_string(), port: 8000 }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct ConvertConfig {
    /// The longest query, in characters.
    pub max_query_chars: usize,
}

impl Default for ConvertConfig {
    fn default() -> Self {
        Self { max_query_chars: 300 }
    }
}

/// `config.toml` in the current directory, and only there.
pub fn default_path() -> PathBuf {
    PathBuf::from("config.toml")
}

/// Reads the file; a file that does not exist gives the defaults.
pub fn load(path: &Path) -> Result<Config> {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Config::default()),
        Err(e) => return Err(format!("cannot read {}: {}", path.display(), e).into()),
    };

    toml::from_str(&text).map_err(|e| format!("{}: {}", path.display(), e).into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_empty_file_is_the_defaults() {
        let config: Config = toml::from_str("").unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8000);
        assert_eq!(config.convert.max_query_chars, 300);
        assert!(config.exceptions.is_empty());
    }

    #[test]
    fn reads_what_is_given_and_defaults_the_rest() {
        let config: Config = toml::from_str(
            "[server]\nport = 9000\n\n[exceptions]\n\"ナース\" = \"nurse\"\n",
        )
        .unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.exceptions.get("ナース").map(String::as_str), Some("nurse"));
    }

    #[test]
    fn a_missing_file_is_the_defaults() {
        let config = load(Path::new("no-such-dir/config.toml")).unwrap();
        assert_eq!(config.server.port, 8000);
    }

    #[test]
    fn a_broken_file_is_an_error_that_names_the_file() {
        let path = std::env::temp_dir().join("pronounce-api-broken-config.toml");
        fs::write(&path, "[server\nport = ").unwrap();
        let error = load(&path).unwrap_err().to_string();
        fs::remove_file(&path).ok();
        assert!(error.contains("pronounce-api-broken-config.toml"), "{error}");
    }
}
