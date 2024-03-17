use serde::{Deserialize, Serialize};
use std::{error::Error, fs, path::Path};
use toml;

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Config {
    pub github_account: String,
    pub dotfiles: Vec<String>,
    pub repo_path: String,
}

pub struct ConfigManager {
    pub config_path: String,
}

impl ConfigManager {
    // Initialize ConfigManager with config file path
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let home = env!("HOME");
        let config_path = format!("{home}/.config/sync-dot-files/sync-dot-files.toml");
        Ok(Self {
            config_path: config_path,
        })
    }

    // Ensure the configuration directory exists, create if it does not
    fn ensure_config_dir(&self) -> Result<(), Box<dyn Error>> {
        let config_dir = Path::new(&self.config_path)
            .parent()
            .ok_or("Unable to determine config directory")?;
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)?;
        }
        Ok(())
    }

    // Load the configuration from the file, return None if file does not exist
    pub fn load(&self) -> Result<Config, Box<dyn Error>> {
        if !Path::new(&self.config_path).exists() {
            return Err("Configuration file does not exist".into());
        }

        let config_str = fs::read_to_string(&self.config_path)?;
        let config: Config = toml::from_str(&config_str)?;
        Ok(config)
    }

    // Save the configuration to the file
    fn save(&self, config: &Config) -> Result<(), Box<dyn Error>> {
        let config_str = toml::to_string(&config)?;

        fs::write(&self.config_path, config_str)?;
        Ok(())
    }

    // Initialize configuration with a GitHub account
    pub fn init(&self, github_account: &str) -> Result<(), Box<dyn Error>> {
        self.ensure_config_dir()?;
        let home = env!("HOME");

        let mut config = match self.load() {
            Ok(config) => config,
            Err(_) => {
                let mut config = Config::default();
                config.repo_path = format!("{home}/.config/sync-dot-files/repo");
                config
            }
        };

        config.github_account = github_account.to_string();
        self.save(&config)?;
        Ok(())
    }

    // Add a dotfile to the configuration
    pub fn add_dotfile(&self, dotfile: &str) -> Result<(), Box<dyn Error>> {
        let mut config = self.load()?;
        config.dotfiles.push(dotfile.to_string());

        self.save(&config)?;
        Ok(())
    }
}
