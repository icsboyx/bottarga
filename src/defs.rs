use std::fmt::Debug;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, create_dir_all, metadata};

use crate::*;

pub(crate) trait PersistentConfig {
    async fn save<'a>(&self, config_dir: Option<&'a str>) -> Result<()>
    where
        Self: Default + Debug + Serialize + for<'de> Deserialize<'de>,
    {
        let file_name = std::any::type_name::<Self>().split("::").last().unwrap().to_owned() + ".toml";
        let file_path = if config_dir.is_some() {
            PathBuf::from(std::env::current_dir().unwrap())
                .join(config_dir.unwrap())
                .join(&file_name)
        } else {
            PathBuf::from(std::env::current_dir().unwrap()).join(&file_name)
        };

        match self.check_file_path(&file_path).await {
            Ok(_) => log!("{} Config path checked successfully", &file_name),
            Err(e) => {
                log_error!("Unable to write to file: {:?}. Error: {}", file_path.parent(), e);
                log_warning!("Proceeding in-memory only, config will not be persistent.");
                return Ok(());
            } // Convert the error into Ok(()) , so the caller can proceed in memory only
        }
        match fs::write(&file_path, toml::to_string_pretty(&self)?).await {
            Ok(_) => {
                log!("{} Config saved successfully: {}", &file_name, file_path.display());
                Result::Ok(())
            }
            Err(e) => {
                log_error!("Unable to write to file: {:?}. Error: {}", file_path.display(), e);
                log_warning!("Proceeding in-memory only, config will not be persistent.");
                Result::Ok(())
            }
        }
    }

    async fn check_file_path(&self, file_path: &PathBuf) -> Result<()> {
        if let Some(parent) = file_path.parent() {
            if !metadata(parent).await.is_ok() {
                log!("Parent directory does not exist, creating: {}", parent.display());
                match create_dir_all(parent).await {
                    Ok(_) => log!("Parent directory created successfully"),
                    Err(e) => {
                        log_error!("Failed to create parent directory: {}", e);
                        return Err(anyhow::anyhow!("Failed to create parent directory: {}", e));
                    }
                }
            }
        }
        Ok(())
    }

    async fn load(config_dir: Option<&str>) -> Self
    where
        Self: Default + Debug + Serialize + for<'de> Deserialize<'de>,
    {
        let file_name = std::any::type_name::<Self>().split("::").last().unwrap().to_owned() + ".toml";
        let file_path = if config_dir.is_some() {
            PathBuf::from(std::env::current_dir().unwrap())
                .join(config_dir.unwrap())
                .join(&file_name)
        } else {
            PathBuf::from(std::env::current_dir().unwrap()).join(&file_name)
        };

        match fs::read_to_string(&file_path).await {
            Ok(content) => {
                log!("{} config loaded successfully: {}", &file_name, file_path.display());
                match toml::from_str(&content) {
                    Ok(config) => config,
                    Err(e) => {
                        log_error!("Unable to parse file: {}. Error: {}", file_path.display(), e.message());
                        log_warning!("Proceeding in-memory only, config will not be persistent.");
                        Self::default()
                    }
                }
            }
            Err(e) => {
                log_error!("Unable to read file: {}. Error: {}", file_path.display(), e);
                log_warning!("Proceeding in-memory only, config will not be persistent.");
                let ret_val = Self::default();
                ret_val.save(config_dir).await.unwrap();
                ret_val
            }
        }
    }
}
