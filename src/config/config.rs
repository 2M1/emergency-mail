use serde::{Deserialize, Serialize};
use std::{env, fs, time::Duration};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IMAPConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub imap: IMAPConfig,
    pub interval: u64,
    pub number_of_copies: u8
}

const ENV_IMAP_HOST: &str = "EM_IMAP_HOST";
const ENV_IMAP_USERNAME: &str = "EM_IMAP_USERNAME";
const ENV_IMAP_PASSWORD: &str = "EM_IMAP_PASSWORD";
const SECONDS_PER_MINUTE: u64 = 60;


impl Config {
    pub fn parse(path: &str) -> Result<Config, String> {
        let config = fs::read_to_string(path)
            .map_err(|_e| format!("couldn't open file at {}", path))?;
        
        let mut config = serde_yaml::from_str::<Config>(&config)
            .map_err(|e| -> String {
                if let Some(location) = e.location() {
                    return format!("couldn't parse yaml: {} at line {}, column {}", e, location.line(), location.column());
                } else {
                    return format!("couldn't parse yaml: {}", e);
                }
            })?;

        if config.imap.host == "" {
            let host = env::var(ENV_IMAP_HOST)
                .map_err(|_e| format!("couldn't get {} from environment", ENV_IMAP_HOST))?;
            let username = env::var(ENV_IMAP_USERNAME)
                .map_err(|_e| format!("couldn't get {} from environment", ENV_IMAP_USERNAME))?;
            let password = env::var(ENV_IMAP_PASSWORD)
                .map_err(|_e| format!("couldn't get {} from environment", ENV_IMAP_PASSWORD))?;
            
            config.imap.password = password;
            config.imap.username = username;
            config.imap.host = host;
        }

        return Ok(config);
    }


    pub fn interval_as_duration(&self) -> Duration {
        return Duration::from_secs(self.interval * SECONDS_PER_MINUTE);
    }
}