use std::fs;
use std::path::{Path, PathBuf};

use crate::ui::style::ColorName;

/// Tri-state preference used for `color` and `ascii`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggle {
    Auto,
    Always,
    Never,
}

impl Toggle {
    pub fn as_str(self) -> &'static str {
        match self {
            Toggle::Auto => "auto",
            Toggle::Always => "always",
            Toggle::Never => "never",
        }
    }

    fn parse(value: &str, key: &str, path: &Path) -> Result<Self, ConfigError> {
        match value {
            "auto" => Ok(Toggle::Auto),
            "always" => Ok(Toggle::Always),
            "never" => Ok(Toggle::Never),
            other => Err(ConfigError::Invalid {
                path: path.to_path_buf(),
                message: format!("{key}: expected auto, always, or never, got \"{other}\""),
            }),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Config {
    pub store_raw: String,
    pub store: PathBuf,
    /// True when `backups` was not present in the file and the default is
    /// derived from the store location.
    pub backups_is_default: bool,
    pub backups_raw: String,
    pub backups: PathBuf,
    pub color: Toggle,
    pub wrap: usize,
    pub undo_depth: usize,
    pub prompt_why: bool,
    pub ascii: Toggle,
    pub current_color: ColorName,
    pub poll_ms: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("could not read config {path}: {source}")]
    Read {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("invalid config {path}: {message}")]
    Invalid { path: PathBuf, message: String },
    #[error("no home directory; set HOME, TRK_CONFIG, or TRK_STORE")]
    NoHome,
}

/// Location of `config.toml`, in the spec's precedence order.
pub fn config_path() -> Option<PathBuf> {
    if let Ok(p) = std::env::var("TRK_CONFIG")
        && !p.is_empty()
    {
        return Some(PathBuf::from(p));
    }
    if let Ok(x) = std::env::var("XDG_CONFIG_HOME")
        && !x.is_empty()
    {
        return Some(Path::new(&x).join("trk").join("config.toml"));
    }
    dirs::home_dir().map(|h| h.join(".config").join("trk").join("config.toml"))
}

/// Default store location used by the first-run flow.
pub fn default_store() -> Result<PathBuf, ConfigError> {
    if let Ok(x) = std::env::var("XDG_DATA_HOME")
        && !x.is_empty()
    {
        return Ok(Path::new(&x).join("trk").join("tasks.trk"));
    }
    dirs::home_dir()
        .map(|h| h.join(".local").join("share").join("trk").join("tasks.trk"))
        .ok_or(ConfigError::NoHome)
}

/// Expand a leading `~` or `$HOME` using the user's home directory.
pub fn expand(value: &str) -> Result<PathBuf, ConfigError> {
    let home = || dirs::home_dir().ok_or(ConfigError::NoHome);
    if value == "~" {
        return home();
    }
    if let Some(rest) = value.strip_prefix("~/") {
        return Ok(home()?.join(rest));
    }
    if value == "$HOME" {
        return home();
    }
    if let Some(rest) = value.strip_prefix("$HOME/") {
        return Ok(home()?.join(rest));
    }
    Ok(PathBuf::from(value))
}

impl Config {
    /// Default backups directory for a given store: a `backups` folder beside
    /// the store.
    fn default_backups(store: &Path) -> PathBuf {
        match store.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.join("backups"),
            _ => PathBuf::from("backups"),
        }
    }

    pub fn from_path(path: &Path) -> Result<(Config, Vec<String>), ConfigError> {
        let text = fs::read_to_string(path).map_err(|source| ConfigError::Read {
            path: path.to_path_buf(),
            source,
        })?;
        Self::parse(&text, path)
    }

    /// Parse config text. Returns the config plus warnings for unknown keys.
    pub fn parse(text: &str, path: &Path) -> Result<(Config, Vec<String>), ConfigError> {
        let table: toml::Table = text.parse().map_err(|e| ConfigError::Invalid {
            path: path.to_path_buf(),
            message: format!("{e}"),
        })?;

        let mut warnings = Vec::new();
        for key in table.keys() {
            if !KNOWN_KEYS.contains(&key.as_str()) {
                warnings.push(format!("unknown config key `{key}` ignored"));
            }
        }

        let store_raw = match table.get("store") {
            Some(toml::Value::String(s)) => s.clone(),
            Some(_) => {
                return Err(ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: "store: expected a string".into(),
                });
            }
            None => {
                return Err(ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: "store: missing required key".into(),
                });
            }
        };
        let store = expand(&store_raw)?;

        let (backups_raw, backups_is_default) = match table.get("backups") {
            Some(toml::Value::String(s)) => (s.clone(), false),
            Some(_) => {
                return Err(ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: "backups: expected a string".into(),
                });
            }
            None => (
                Config::default_backups(&store)
                    .to_string_lossy()
                    .into_owned(),
                true,
            ),
        };
        let backups = expand(&backups_raw)?;

        let color = match table.get("color") {
            Some(toml::Value::String(s)) => Toggle::parse(s, "color", path)?,
            Some(_) => {
                return Err(ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: "color: expected a string".into(),
                });
            }
            None => Toggle::Auto,
        };

        let ascii = match table.get("ascii") {
            Some(toml::Value::String(s)) => Toggle::parse(s, "ascii", path)?,
            Some(_) => {
                return Err(ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: "ascii: expected a string".into(),
                });
            }
            None => Toggle::Auto,
        };

        let current_color = match table.get("current_color") {
            Some(toml::Value::String(s)) => {
                ColorName::parse(s).ok_or_else(|| ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: format!(
                        "current_color: expected one of {}, got \"{s}\"",
                        ColorName::NAMES.join(", ")
                    ),
                })?
            }
            Some(_) => {
                return Err(ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: "current_color: expected a string".into(),
                });
            }
            None => ColorName::default(),
        };

        let wrap = match table.get("wrap") {
            Some(toml::Value::Integer(n)) => (*n).clamp(40, 200) as usize,
            Some(_) => {
                return Err(ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: "wrap: expected an integer".into(),
                });
            }
            None => 80,
        };

        let undo_depth = match table.get("undo_depth") {
            Some(toml::Value::Integer(n)) if *n >= 0 => *n as usize,
            Some(_) => {
                return Err(ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: "undo_depth: expected a non-negative integer".into(),
                });
            }
            None => 50,
        };

        let prompt_why = match table.get("prompt_why") {
            Some(toml::Value::Boolean(b)) => *b,
            Some(_) => {
                return Err(ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: "prompt_why: expected a boolean".into(),
                });
            }
            None => true,
        };

        let poll_ms = match table.get("poll_ms") {
            Some(toml::Value::Integer(n)) if *n > 0 => *n as u64,
            Some(_) => {
                return Err(ConfigError::Invalid {
                    path: path.to_path_buf(),
                    message: "poll_ms: expected a positive integer".into(),
                });
            }
            None => 500,
        };

        let config = Config {
            store_raw,
            store,
            backups_is_default,
            backups_raw,
            backups,
            color,
            wrap,
            undo_depth,
            prompt_why,
            ascii,
            current_color,
            poll_ms,
        };
        Ok((config, warnings))
    }

    /// Build a config from a `TRK_STORE` override with every other key at its
    /// default (used when no config file exists).
    pub fn with_store(store_raw: String) -> Result<Config, ConfigError> {
        let store = expand(&store_raw)?;
        let backups = Config::default_backups(&store);
        Ok(Config {
            store_raw,
            store,
            backups_is_default: true,
            backups_raw: backups.to_string_lossy().into_owned(),
            backups,
            color: Toggle::Auto,
            wrap: 80,
            undo_depth: 50,
            prompt_why: true,
            ascii: Toggle::Auto,
            current_color: ColorName::default(),
            poll_ms: 500,
        })
    }

    /// Apply a `TRK_STORE` override to a loaded config.
    pub fn override_store(&mut self, store_raw: String) -> Result<(), ConfigError> {
        let store = expand(&store_raw)?;
        self.store = store;
        self.store_raw = store_raw;
        if self.backups_is_default {
            self.backups = Config::default_backups(&self.store);
            self.backups_raw = self.backups.to_string_lossy().into_owned();
        }
        Ok(())
    }

    /// Render canonical config text (all keys present).
    pub fn to_toml(&self) -> String {
        format!(
            "# trk configuration\n\
             store = {store:?}\n\
             backups = {backups:?}\n\
             color = \"{color}\"\n\
             wrap = {wrap}\n\
             undo_depth = {undo}\n\
             prompt_why = {why}\n\
             ascii = \"{ascii}\"\n\
             current_color = \"{current_color}\"\n\
             poll_ms = {poll}\n",
            store = self.store_raw,
            backups = self.backups_raw,
            color = self.color.as_str(),
            wrap = self.wrap,
            undo = self.undo_depth,
            why = self.prompt_why,
            ascii = self.ascii.as_str(),
            current_color = self.current_color.as_str(),
            poll = self.poll_ms,
        )
    }
}

const KNOWN_KEYS: &[&str] = &[
    "store",
    "backups",
    "color",
    "wrap",
    "undo_depth",
    "prompt_why",
    "ascii",
    "current_color",
    "poll_ms",
];
