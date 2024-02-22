use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::{env, fs, str::FromStr, time::Duration};

#[derive(Debug, Serialize, Deserialize, Clone, Default, Eq, PartialEq)]
pub enum IMAPModes {
    #[serde(alias = "idle", alias = "IDLE")]
    #[default]
    Idle,
    #[serde(alias = "poll", alias = "POLL")]
    Poll,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IMAPModeDescription {
    pub method: IMAPModes,
    pub interval: u64, // in minutes or seconds depending on the mode
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct IMAPConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub mode: IMAPModeDescription,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrintingConfig {
    pub printer: Option<String>, // None indicates, that the default system printer should be used
    pub min_copies: u8,
    pub max_copies: Option<u8>,
    pub additional_copies: Option<u8>,
    pub amt: u8,
    pub sumatra_path: String,
    pub disable: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub imap: IMAPConfig,
    pub printing: PrintingConfig,
    pub pdf_save_path: Option<String>,
}

const ENV_IMAP_HOST: &str = "EM_IMAP_HOST";
const ENV_IMAP_USERNAME: &str = "EM_IMAP_USERNAME";
const ENV_IMAP_PASSWORD: &str = "EM_IMAP_PASSWORD";
const SECONDS_PER_MINUTE: u64 = 60;
pub const IMAP_IDLE_DEFAULT_INTERVAL: u64 = 29; // as per RFC 2177
pub const IMAP_IDLE_MAX_INTERVAL: u64 = 29; // as per RFC 2177

impl Config {
    pub fn parse(path: &str) -> Result<Config, String> {
        let config =
            fs::read_to_string(path).map_err(|_e| format!("couldn't open file at {}", path))?;
        return Config::from_str(&config);
    }

    pub fn interval_as_duration(&self) -> Duration {
        return self.imap.mode.interval_as_duration();
    }
}

impl IMAPModeDescription {
    pub fn interval_as_duration(&self) -> Duration {
        match self.method {
            IMAPModes::Poll => Duration::from_secs(self.interval),
            IMAPModes::Idle => Duration::from_secs(self.interval * SECONDS_PER_MINUTE),
        }
    }
}

impl Default for IMAPModeDescription {
    fn default() -> Self {
        return IMAPModeDescription {
            method: IMAPModes::Idle,
            interval: IMAP_IDLE_DEFAULT_INTERVAL,
        };
    }
}

impl PrintingConfig {
    pub fn disabled(&self) -> bool {
        return self.disable.unwrap_or(false);
    }
}

impl FromStr for Config {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut config = serde_yaml::from_str::<Config>(&s).map_err(|e| -> String {
            return format!("couldn't parse yaml: {}", e);
        })?;

        // imap required field resolution
        if config.imap.host == "" {
            let host = env::var(ENV_IMAP_HOST)
                .map_err(|_e| format!("couldn't get {} from environment", ENV_IMAP_HOST))?;
            config.imap.host = host;
            debug!("acquired imap host from environment: {}", config.imap.host);
        }

        if config.imap.password.is_empty() {
            let password = env::var(ENV_IMAP_PASSWORD)
                .map_err(|_e| format!("couldn't get {} from environment", ENV_IMAP_PASSWORD))?;

            config.imap.password = password;
            debug!("acquired imap password from environment");

            if config.imap.username.is_empty() {
                // only allow empty username, if password is also empty (makes no sense otherwise)
                let username = env::var(ENV_IMAP_USERNAME)
                    .map_err(|_e| format!("couldn't get {} from environment", ENV_IMAP_USERNAME))?;
                config.imap.username = username;
                debug!("acquired imap username from environment");
            }
        }

        // imap sanity checks
        if config.imap.mode.interval == 0 {
            return Err("interval for IMAP mode must be greater than 0".to_string());
        }

        if config.imap.mode.method == IMAPModes::Idle
            && config.imap.mode.interval > IMAP_IDLE_MAX_INTERVAL
        {
            return Err("Interval for IDLE outside of RFC 2177 specification!".to_string());
        }

        // printing sanity checks
        if config.printing.disabled() {
            if cfg!(not(debug_assertions)) && config.pdf_save_path.is_none() {
                // during debug, a test.pdf file is always saved to the current directory
                return Err("printing is disabled, but no pdf save path is set".to_string());
            }
            info!("printing is disabled!");
        }

        return Ok(config);
    }
}
