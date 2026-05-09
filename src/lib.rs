pub mod errors;
mod path;

pub use errors::{ConfigError, Result};
use serde::{Serialize, de::DeserializeOwned};
use std::fs;
use std::path::PathBuf;

/// Supported serialization formats for the configuration file.
#[derive(Clone, Copy, Debug)]
pub enum Format {
    Json,
    Toml,
}

/// Manages the loading and saving of application configuration.
pub struct ConfigManager {
    file_path: PathBuf,
    format: Format,
}

/// A builder for creating a [`ConfigManager`] with specific platform identifiers.
pub struct ConfigManagerBuilder {
    qualifier: String,
    organization: String,
    application: String,
    filename: String,
    format: Format,
}

impl ConfigManager {
    /// Starts building a `ConfigManager` with your application's identity.
    ///
    /// # Parameters
    /// * `qualifier`: The reverse domain name (e.g., "com").
    /// * `organization`: The name of your organization.
    /// * `application`: The name of your application.
    #[must_use]
    pub fn builder(qualifier: &str, organization: &str, application: &str) -> ConfigManagerBuilder {
        ConfigManagerBuilder {
            qualifier: qualifier.to_string(),
            organization: organization.to_string(),
            application: application.to_string(),
            filename: "config".to_string(),
            format: Format::Json,
        }
    }

    /// Loads the configuration from disk.
    ///
    /// If the file does not exist, it returns `T::default()`.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Io`] if the file exists but cannot be read.
    /// Returns [`ConfigError::Deserialization`] if the file content is not valid for the chosen [`Format`].
    /// Returns [`ConfigError::FeatureMissing`] if the required serialization feature is not enabled.
    pub fn load_or_default<T: DeserializeOwned + Default>(&self) -> Result<T> {
        if !self.file_path.exists() {
            return Ok(T::default());
        }

        let data = fs::read_to_string(&self.file_path).map_err(|e| ConfigError::Io {
            path: self.file_path.clone(),
            source: e,
        })?;

        match self.format {
            Format::Json => {
                #[cfg(feature = "json")]
                {
                    serde_json::from_str(&data)
                        .map_err(|e| ConfigError::Deserialization(e.to_string()))
                }
                #[cfg(not(feature = "json"))]
                {
                    Err(ConfigError::FeatureMissing("json".to_string()));
                }
            }
            Format::Toml => {
                #[cfg(feature = "toml")]
                {
                    toml::from_str(&data).map_err(|e| ConfigError::Deserialization(e.to_string()))
                }
                #[cfg(not(feature = "toml"))]
                {
                    Err(ConfigError::FeatureMissing("toml".to_string()))
                }
            }
        }
    }

    /// Saves the configuration to disk using an atomic write pattern.
    ///
    /// This method creates a temporary file and renames it to the target path to prevent
    /// file corruption in the event of a crash or power failure.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Io`] if the directory cannot be created, the temporary file cannot
    /// be written, or the rename operation fails.
    /// Returns [`ConfigError::Serialization`] if the data structure cannot be serialized.
    /// Returns [`ConfigError::FeatureMissing`] if the required serialization feature is not enabled.
    pub fn save<T: Serialize>(&self, config: &T) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            fs::create_dir_all(parent).map_err(|e| ConfigError::Io {
                path: parent.to_path_buf(),
                source: e,
            })?;
        }

        let content = match self.format {
            Format::Json => {
                #[cfg(feature = "json")]
                {
                    serde_json::to_string_pretty(config)
                        .map_err(|e| ConfigError::Serialization(e.to_string()))?
                }
                #[cfg(not(feature = "json"))]
                {
                    Err(ConfigError::FeatureMissing("json".to_string()))
                }
            }
            Format::Toml => {
                #[cfg(feature = "toml")]
                {
                    toml::to_string_pretty(config)
                        .map_err(|e| ConfigError::Serialization(e.to_string()))?
                }
                #[cfg(not(feature = "toml"))]
                {
                    return Err(ConfigError::FeatureMissing("toml".to_string()));
                }
            }
        };

        // Atomic write: Write to .tmp, then rename
        let mut tmp_path = self.file_path.clone();
        tmp_path.set_extension("tmp");

        fs::write(&tmp_path, content).map_err(|e| ConfigError::Io {
            path: tmp_path.clone(),
            source: e,
        })?;

        fs::rename(&tmp_path, &self.file_path).map_err(|e| ConfigError::Io {
            path: self.file_path.clone(),
            source: e,
        })?;

        Ok(())
    }
}

impl ConfigManagerBuilder {
    /// Sets the filename for the configuration (default is "config").
    #[must_use]
    pub fn with_filename(mut self, filename: &str) -> Self {
        self.filename = filename.to_string();
        self
    }

    /// Sets the serialization format.
    #[must_use]
    pub const fn with_format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }

    /// Consumes the builder and returns a [`ConfigManager`].
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::UnsupportedPlatform`] if the OS-specific configuration
    /// directory cannot be resolved.
    pub fn build(self) -> Result<ConfigManager> {
        let base_dir = path::get_config_dir(&self.qualifier, &self.organization, &self.application)
            .ok_or(ConfigError::UnsupportedPlatform)?;

        let ext = match self.format {
            Format::Json => "json",
            Format::Toml => "toml",
        };

        Ok(ConfigManager {
            file_path: base_dir.join(format!("{}.{}", self.filename, ext)),
            format: self.format,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::{Deserialize, Serialize};
    use tempfile::tempdir;

    #[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
    struct TestConfig {
        name: String,
        version: u32,
        enabled: bool,
    }

    #[cfg(feature = "json")]
    #[test]
    fn test_save_and_load_json() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("config.json");

        let manager = ConfigManager {
            file_path: file_path.clone(),
            format: Format::Json,
        };

        let data = TestConfig {
            name: "TestApp".into(),
            version: 1,
            enabled: true,
        };

        // 1. Save data
        manager.save(&data).expect("Failed to save config");

        // 2. Load data
        let loaded: TestConfig = manager.load_or_default().expect("Failed to load config");

        assert_eq!(data, loaded);
        assert!(file_path.exists());
    }

    #[test]
    fn test_load_default_when_missing() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("non_existent.json");

        let manager = ConfigManager {
            file_path,
            format: Format::Json,
        };

        let loaded: TestConfig = manager.load_or_default().expect("Failed to load");
        assert_eq!(loaded, TestConfig::default());
    }

    #[test]
    #[cfg(feature = "toml")]
    fn test_save_and_load_toml() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("config.toml");

        let manager = ConfigManager {
            file_path,
            format: Format::Toml,
        };

        let data = TestConfig {
            name: "TomlApp".into(),
            version: 2,
            enabled: false,
        };

        manager.save(&data).expect("Save failed");
        let loaded: TestConfig = manager.load_or_default().expect("Load failed");

        assert_eq!(data, loaded);
    }

    #[test]
    fn test_atomic_write_integrity() {
        let dir = tempdir().expect("Failed to create temp dir");
        let file_path = dir.path().join("atomic.json");

        let manager = ConfigManager {
            file_path: file_path.clone(),
            format: Format::Json,
        };

        let data = TestConfig::default();

        // Ensure that even if we save multiple times, the .tmp file is cleaned up
        manager.save(&data).unwrap();
        manager.save(&data).unwrap();

        let tmp_path = file_path.with_extension("tmp");
        assert!(
            !tmp_path.exists(),
            "Temporary file should have been renamed/removed"
        );
        assert!(file_path.exists(), "Final config file should exist");
    }
}
