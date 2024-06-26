use std::{env, str::FromStr};

use crate::config::config::IMAPModes::{Idle, Poll};
use crate::config::config::IMAP_IDLE_DEFAULT_INTERVAL;
use crate::config::Config;

#[cfg(test)]
const TEST_FULL_CONFIG: &str = include_str!("../../examples/config_full.yaml");

#[test]
fn test_from_str() {
    let config = Config::from_str(TEST_FULL_CONFIG).unwrap();
    assert_eq!(config.imap.host, "imap.gmail.com");
    assert_eq!(config.imap.port, 993);
    assert_eq!(config.imap.username, "abc");
    assert_eq!(config.imap.password, "def");
    assert_eq!(config.imap.mode.interval, 25);
    assert_eq!(config.imap.mode.method, Poll);
    assert_eq!(config.printing.min_copies, 2);
    assert_eq!(config.printing.max_copies, Some(5));
    assert_eq!(
        config.printing.printer,
        Some("HP_LaserJet_500_Pro".to_string())
    );
    assert_eq!(config.printing.amt, 1);
    assert_eq!(config.printing.disabled(), false);
    assert_eq!(config.printing.disable, Some(false))
}

#[test]
fn test_parse_file() {
    let config = Config::parse("examples/config_full.yaml").unwrap();
    assert_eq!(config.imap.host, "imap.gmail.com");
    assert_eq!(config.imap.port, 993);
    assert_eq!(config.imap.username, "abc");
    assert_eq!(config.imap.password, "def");
    assert_eq!(config.imap.mode.interval, 25);
    assert_eq!(config.imap.mode.method, Poll);
    assert_eq!(config.printing.min_copies, 2);
    assert_eq!(config.printing.max_copies, Some(5));
    assert_eq!(config.printing.additional_copies, Some(1));
    assert_eq!(
        config.printing.printer,
        Some("HP_LaserJet_500_Pro".to_string())
    );
    assert_eq!(config.printing.amt, 1);
    assert_eq!(
        config.printing.sumatra_path,
        "C:\\Users\\Markus\\AppData\\Local\\SumatraPDF\\SumatraPDF.exe".to_string()
    );
    assert_eq!(config.printing.disabled(), false);
    assert_eq!(config.printing.disable, Some(false))
}

#[test]
fn test_parse_file_minmal_config() {
    env::set_var("EM_IMAP_HOST", "host");
    env::set_var("EM_IMAP_USERNAME", "user");
    env::set_var("EM_IMAP_PASSWORD", "pass");

    let config = Config::parse("examples/config.yaml").unwrap();
    assert_eq!(config.imap.host, "host"); // should be pulled from environment
    assert_eq!(config.imap.port, 993);
    assert_eq!(config.imap.username, "user"); // as should this
    assert_eq!(config.imap.password, "pass"); // and this
    assert_eq!(config.imap.mode.interval, IMAP_IDLE_DEFAULT_INTERVAL); // default value, as not set in file
    assert_eq!(config.imap.mode.method, Idle); // default value, as not set in file
    assert_eq!(config.printing.min_copies, 1);
    assert_eq!(config.printing.max_copies, None);
    assert_eq!(
        config.printing.printer,
        Some("HPE76479 (HP OfficeJet Pro 8020 series)".to_string())
    );
    assert_eq!(config.printing.amt, 1);
    assert_eq!(
        config.printing.sumatra_path,
        "C:\\Users\\Markus\\AppData\\Local\\SumatraPDF\\SumatraPDF.exe".to_string()
    );
    assert_eq!(config.printing.additional_copies, None);
    assert_eq!(config.printing.disable, None);
    assert_eq!(config.printing.disabled(), false);

    env::remove_var("EM_IMAP_HOST");
    env::remove_var("EM_IMAP_USERNAME");
    env::remove_var("EM_IMAP_PASSWORD");
}
