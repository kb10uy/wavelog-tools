use std::{
    collections::HashMap,
    env,
    fs::read_to_string,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use etcetera::{BaseStrategy, choose_base_strategy};
use secrecy::SecretString;
use serde::Deserialize;
use url::Url;

use crate::wavelog::WavelogClient;

const APP_NAME: &str = "wavelog-tools";
const CONFIG_FILENAME: &str = "config.toml";
const TOKEN_ENV: &str = "WAVELOG_TOKEN";

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub wavelog: Option<WavelogConfig>,

    #[serde(default)]
    pub operators: HashMap<String, OperatorConfig>,

    #[serde(skip)]
    path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WavelogConfig {
    pub url: Url,
    pub token: Option<SecretString>,
    pub token_file: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OperatorConfig {
    pub name: String,
}

impl Config {
    pub fn default_path() -> Result<PathBuf> {
        let strategy = choose_base_strategy().context("failed to determine home directory")?;
        Ok(strategy.config_dir().join(APP_NAME).join(CONFIG_FILENAME))
    }

    pub fn load(explicit_path: Option<&Path>) -> Result<Config> {
        let path = match explicit_path {
            Some(path) => path.to_path_buf(),
            None => Config::default_path()?,
        };

        let text = match read_to_string(&path) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound && explicit_path.is_none() => {
                return Ok(Config {
                    path,
                    ..Default::default()
                });
            }
            Err(e) => {
                return Err(e).with_context(|| format!("failed to read {}", path.display()));
            }
        };
        let config: Config =
            toml::from_str(&text).with_context(|| format!("failed to parse {}", path.display()))?;
        Ok(Config { path, ..config })
    }

    /// Returns the path this config was loaded from, whether or not the file exists.
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn sibling_file(&self, filename: &str) -> Option<PathBuf> {
        let path = self.path.parent()?.join(filename);
        path.is_file().then_some(path)
    }

    pub fn wavelog_client(&self) -> Result<Option<WavelogClient>> {
        let Some(wavelog) = &self.wavelog else {
            return Ok(None);
        };
        let token = wavelog.resolve_token()?;
        Ok(Some(WavelogClient::new(&wavelog.url, token)?))
    }
}

impl WavelogConfig {
    fn resolve_token(&self) -> Result<SecretString> {
        if let Some(token) = &self.token {
            return Ok(token.clone());
        }
        if let Some(path) = &self.token_file {
            let token = read_to_string(path)
                .with_context(|| format!("failed to read {}", path.display()))?;
            return Ok(SecretString::from(token.trim()));
        }
        match env::var(TOKEN_ENV) {
            Ok(token) => Ok(SecretString::from(token.trim())),
            Err(_) => bail!(
                "Wavelog token not found; set wavelog.token or wavelog.token_file in config, or {TOKEN_ENV}"
            ),
        }
    }
}
