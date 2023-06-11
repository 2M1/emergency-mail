use std::str::FromStr;

use crate::config::Config;

#[cfg(test)]
const TEST_FULL_CONFIG: &'static str = include_str!("../../examples/config_full.yaml");

#[test]
fn test_from_str() {
    let config = Config::from_str(TEST_FULL_CONFIG).unwrap();
    assert_eq!(config.imap.host, "imap.gmail.com");
    assert_eq!(config.imap.port, 993);
    assert_eq!(config.imap.username, "abc");
    assert_eq!(config.imap.password, "def");
    assert_eq!(config.interval, 25);
    assert_eq!(config.printing.min_copies, 2);
    assert_eq!(
        config.printing.printer,
        Some("HP_LaserJet_500_Pro".to_string())
    );
    assert_eq!(config.printing.amt, 1);
}

#[test]
fn test_parse_file() {
    let config = Config::parse("examples/config_full.yaml").unwrap();
    assert_eq!(config.imap.host, "imap.gmail.com");
    assert_eq!(config.imap.port, 993);
    assert_eq!(config.imap.username, "abc");
    assert_eq!(config.imap.password, "def");
    assert_eq!(config.interval, 25);
    assert_eq!(config.printing.min_copies, 2);
    assert_eq!(
        config.printing.printer,
        Some("HP_LaserJet_500_Pro".to_string())
    );
    assert_eq!(config.printing.amt, 1);
}
