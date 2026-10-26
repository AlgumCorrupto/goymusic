//! One shared TOML file, many typed sections.
//!
//! ```ignore
//! #[config_secton("improviser")]
//! #[derive(Default)]
//! pub struct ImproviserConfig { pub tempo: f32, pub octave: u8 }
//!
//! let path = Config::default_path("goymusic");
//! let mut cfg = Config::load(&path)?;
//! let mut ic: ImproviserConfig = cfg.load_into()?;
//! ic.tempo = 175.0;
//! cfg.store(&ic)?;
//! cfg.save(&path)?;
//! ```

// Lets the generated `::config::...` paths resolve inside this crate too (tests).
extern crate self as config;

pub use config_section_derive::config_section;
pub use serde;
pub use toml;

use serde::{de::DeserializeOwned, Serialize};
use std::{
    fmt, fs, io,
    path::{Path, PathBuf},
};

/// Implemented by `#[configurable("path")]`. `PATH` is the dotted location in the file.
pub trait ConfigSection: Serialize + DeserializeOwned + Default {
    const PATH: &'static str;
}

#[derive(Debug)]
pub enum ConfigError {
    Io(io::Error),
    Parse(toml::de::Error),
    Section { path: &'static str, source: toml::de::Error },
    Serialize { path: String, source: toml::ser::Error },
    /// A parent key on the path exists but is not a table.
    Conflict(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "config I/O error: {e}"),
            Self::Parse(e) => write!(f, "invalid TOML: {e}"),
            Self::Section { path, source } => write!(f, "[{path}]: {source}"),
            Self::Serialize { path, source } => write!(f, "cannot serialize [{path}]: {source}"),
            Self::Conflict(path) => write!(f, "[{path}]: a parent key is not a table"),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

/// The whole file, kept as a table so unknown sections survive a save.
pub struct Config {
    pub root: toml::Table,
    path: PathBuf,
}

impl Config {
    pub fn empty() -> Self {
        Self { root: toml::Table::new(), path: PathBuf::default() }
    }

    pub fn path(&self) -> &PathBuf { &self.path }

    /// `$XDG_CONFIG_HOME/<app>/config.toml`, falling back to `~/.config/<app>/config.toml`.
    pub fn default_path(app: &str) -> PathBuf {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".config")))
            .unwrap_or_else(|| PathBuf::from("."));
        base.join(app).join("config.toml")
    }

    /// A missing file is not an error: you get an empty config (all defaults).
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        
        match fs::read_to_string(path) {
            Ok(text) => Ok(Self { root: text.parse().map_err(ConfigError::Parse)?, path: path.into() }),
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                Ok(Self { root: toml::Table::new(), path: path.into() })
            }
            Err(e) => Err(ConfigError::Io(e)),

        }
    }

    /// "tts.fish" -> the `[tts.fish]` value, if present.
    pub fn section(&self, path: &str) -> Option<&toml::Value> {
        let mut parts = path.split('.');
        let mut cur = self.root.get(parts.next()?)?;
        for p in parts {
            cur = cur.get(p)?;
        }
        Some(cur)
    }

    /// Builds `T` from its section. Missing section = `T::default()`.
    pub fn load_into<T: ConfigSection>(&self) -> Result<T, ConfigError> {
        match self.section(T::PATH) {
            None => Ok(T::default()),
            Some(v) => v
                .clone()
                .try_into()
                .map_err(|source| ConfigError::Section { path: T::PATH, source }),
        }
    }

    pub fn set_value(&mut self, path: &str, value: toml::Value) -> Result<(), ConfigError> {
        let mut parts: Vec<&str> = path.split('.').collect();
        let last = parts.pop().ok_or_else(|| ConfigError::Conflict(path.into()))?;

        let mut table = &mut self.root;
        for p in parts {
            table = table
                .entry(p)
                .or_insert_with(|| toml::Value::Table(toml::Table::new()))
                .as_table_mut()
                .ok_or_else(|| ConfigError::Conflict(path.into()))?;
        }
        table.insert(last.to_string(), value);
        Ok(())
    }
    
    /// Serializes `value` and stores it at a dotted path, creating parent tables as needed.
    /// In-memory only; call `save` to persist.
    pub fn set<T: Serialize + ?Sized>(&mut self, path: &str, value: &T) -> Result<(), ConfigError> {
        let v = toml::Value::try_from(value)
            .map_err(|source| ConfigError::Serialize { path: path.into(), source })?;
        self.set_value(path, v)
    }

    /// `store` becomes a thin wrapper, so the path-walking code exists once.
    pub fn store<T: ConfigSection>(&mut self, value: &T) -> Result<(), ConfigError> {
        self.set(T::PATH, value)
    }


    /// Writes the file atomically (temp file + rename). On Unix it is created `0600`,
    /// since it may hold an API key.
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let text = toml::to_string_pretty(&self.root)
            .map_err(|source| ConfigError::Serialize { path: "(root)".into(), source })?;
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir)?;
        }

        let tmp = path.with_extension("toml.tmp");
        {
            use std::io::Write;
            let mut opts = fs::OpenOptions::new();
            opts.write(true).create(true).truncate(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                opts.mode(0o600);
            }
            let mut f = opts.open(&tmp)?;
            f.write_all(text.as_bytes())?;
        }
        fs::rename(&tmp, path)?;
        Ok(())
    }

    /// Reads and deserializes the value at a dotted path.
    /// Missing = `None`. Present but the wrong type = logs the error and returns `None`.
    pub fn get<T: DeserializeOwned>(&self, path: &str) -> Option<T> {
        let v = self.section(path)?;
        match v.clone().try_into() {
            Ok(t) => Some(t),
            Err(e) => {
                eprintln!("config: [{path}]: {e}");
                None
            }
        }
    }
}

pub trait Configurable {
    /// Builds `Self` from its saved section. Missing or broken section = defaults.
    fn load(config: &Config) -> Self
    where
        Self: Sized;

    fn store(&self, config: &mut Config);
}


#[cfg(test)]
mod tests {
    use super::*;

    #[config_section("demo.inner")]
    #[derive(Default, Debug, PartialEq)]
    pub struct Demo {
        pub tempo: f32,
        pub name: String,
        pub list: Vec<String>,
    }

    #[test]
    fn missing_section_gives_default() {
        let cfg = Config::empty();
        assert_eq!(cfg.load_into::<Demo>().unwrap(), Demo::default());
    }

    #[test]
    fn roundtrip_nested_path() {
        let mut cfg = Config::empty();
        let d = Demo { tempo: 175.0, name: "x".into(), list: vec!["a".into(), "b".into()] };
        cfg.store(&d).unwrap();
        assert_eq!(cfg.section("demo.inner").unwrap().get("tempo").unwrap().as_float(), Some(175.0));
        assert_eq!(cfg.load_into::<Demo>().unwrap(), d);
    }

    #[test]
    fn partial_section_fills_defaults() {
        let cfg = Config { root: "[demo.inner]\ntempo = 90.0\n".parse().unwrap(), path: PathBuf::default() };
        let d: Demo = cfg.load_into().unwrap();
        assert_eq!(d.tempo, 90.0);
        assert_eq!(d.name, "");
    }

    #[test]
    fn store_keeps_other_sections() {
        let mut cfg = Config { root: "[other]\nkeep = 1\n".parse().unwrap(), path: PathBuf::default() };
        cfg.store(&Demo::default()).unwrap();
        assert!(cfg.section("other.keep").is_some());
    }
}