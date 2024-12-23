use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::{env, fs::File, path::PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub auth: AuthConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthConfig {
    pub pk: String,
    pub sk: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub port: u16,
    pub db_url: String,
    pub base_dir: PathBuf,
}

impl AppConfig {
    pub fn load() -> Result<Self> {
        // read from ./app.yml, or /etc/config/app.yml, or from env CHAT_CONFIG
        let res = match (
            File::open("chat.yml"),
            File::open("/etc/config/chat.yml"),
            env::var("CHAT_CONFIG"),
        ) {
            (Ok(file), _, _) => serde_yaml::from_reader(file),
            (_, Ok(file), _) => serde_yaml::from_reader(file),
            (_, _, Ok(env)) => serde_yaml::from_reader(File::open(env)?),
            _ => bail!("Config file not found"),
        };
        Ok(res?)
    }
}
