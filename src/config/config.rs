use log::info;
use serde::{Deserialize, Serialize};
use std::{env, fs, str::FromStr, time::Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IMAPConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrintingConfig {
    pub printer: Option<String>, // None indicates, that the default system printer should be used
    pub min_copies: u8,
    pub amt: u8,
    pub sumatra_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub imap: IMAPConfig,
    pub interval: u64,
    pub printing: PrintingConfig,
}

const ENV_IMAP_HOST: &str = "EM_IMAP_HOST";
const ENV_IMAP_USERNAME: &str = "EM_IMAP_USERNAME";
const ENV_IMAP_PASSWORD: &str = "EM_IMAP_PASSWORD";
const SECONDS_PER_MINUTE: u64 = 60;

impl Config {
    pub fn parse(path: &str) -> Result<Config, String> {
        let config =
            fs::read_to_string(path).map_err(|_e| format!("couldn't open file at {}", path))?;
        return Config::from_str(&config);
    }

    pub fn interval_as_duration(&self) -> Duration {
        return Duration::from_secs(self.interval * SECONDS_PER_MINUTE);
    }
}

impl FromStr for Config {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut config = serde_yaml::from_str::<Config>(&s).map_err(|e| -> String {
            return format!("couldn't parse yaml: {}", e);
        })?;

        if config.imap.host == "" {
            let host = env::var(ENV_IMAP_HOST)
                .map_err(|_e| format!("couldn't get {} from environment", ENV_IMAP_HOST))?;
            config.imap.host = host;
            info!("acquired imap host from environment: {}", config.imap.host);
        }

        if config.imap.password.is_empty() {
            let password = env::var(ENV_IMAP_PASSWORD)
                .map_err(|_e| format!("couldn't get {} from environment", ENV_IMAP_PASSWORD))?;

            config.imap.password = password;
            info!("acquired imap password from environment");

            if config.imap.username.is_empty() {
                // only allow empty username, if password is also empty (makes no sense otherwise)
                let username = env::var(ENV_IMAP_USERNAME)
                    .map_err(|_e| format!("couldn't get {} from environment", ENV_IMAP_USERNAME))?;
                config.imap.username = username;
                info!("acquired imap username from environment");
            }
        }

        return Ok(config);
    }
}
