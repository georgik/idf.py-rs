use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct SdkConfig {
    pub target: Option<String>,
    pub settings: HashMap<String, String>,
}

impl SdkConfig {
    pub fn load_from_file(path: &Path) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            Ok(Self::parse_sdkconfig(&content)?)
        } else {
            Ok(Self {
                target: None,
                settings: HashMap::new(),
            })
        }
    }

    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content = self.to_sdkconfig_format();
        fs::write(path, content)?;
        Ok(())
    }

    fn parse_sdkconfig(content: &str) -> Result<Self> {
        let mut settings = HashMap::new();
        let mut target = None;

        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                if key == "CONFIG_IDF_TARGET" {
                    target = Some(value.trim_matches('"').to_string());
                }

                settings.insert(key.to_string(), value.to_string());
            }
        }

        Ok(Self { target, settings })
    }

    fn to_sdkconfig_format(&self) -> String {
        let mut lines = Vec::new();

        // Add header comment
        lines.push("# ESP-IDF Configuration".to_string());
        lines.push("".to_string());

        // Sort keys for consistent output
        let mut sorted_keys: Vec<_> = self.settings.keys().collect();
        sorted_keys.sort();

        for key in sorted_keys {
            if let Some(value) = self.settings.get(key) {
                lines.push(format!("{}={}", key, value));
            }
        }

        lines.join("\n")
    }

    pub fn set_target(&mut self, target: &str) {
        self.target = Some(target.to_string());
        self.settings
            .insert("CONFIG_IDF_TARGET".to_string(), format!("\"{}\"", target));
    }

    pub fn get_target(&self) -> Option<&String> {
        self.target.as_ref()
    }
}

pub fn get_sdkconfig_path(project_dir: &Path) -> PathBuf {
    project_dir.join("sdkconfig")
}

pub fn get_sdkconfig_defaults_path(project_dir: &Path) -> PathBuf {
    project_dir.join("sdkconfig.defaults")
}

pub fn load_project_config(project_dir: &Path) -> Result<SdkConfig> {
    let sdkconfig_path = get_sdkconfig_path(project_dir);
    SdkConfig::load_from_file(&sdkconfig_path)
}

pub fn save_project_config(project_dir: &Path, config: &SdkConfig) -> Result<()> {
    let sdkconfig_path = get_sdkconfig_path(project_dir);
    config.save_to_file(&sdkconfig_path)
}
