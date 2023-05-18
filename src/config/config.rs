use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct SMTPConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub smtp: SMTPConfig,
    pub interval: u64,
    pub number_of_copies: u8
}

impl Config {
    pub fn parse(path: &str) -> Result<Config, String> {
        let config = std::fs::read_to_string(path)
            .map_err(|_e| format!("couldn't open file at {}", path))?;
        
        let config = serde_yaml::from_str::<Config>(&config)
            .map_err(|_e| "couldn't parse config".to_string());
        
        return config;
    }
}