use std::str::FromStr;

use crate::config::Config;

#[cfg(test)]
const TEST_FULL_CONFIG: &str = r#"imap:
  host: "imap.gmail.com" # leave empty to use the environment variable EM_SMTP_HOST
  port: 993
  username: "" # leave empty to use the environment variable EM_SMTP_USERNAME
  password: "" # leave empty to use the environment variable EM_SMTP_PASSWORD
interval: 25 # in minutes
printing:
  min_copies: 2 # number of duplicate copies to be printed
  printer: null # "HP_LaserJet_400_M401dn" # printer name // TODO: add instructions on how to get the printer name
  amt: 1 # AMT number (Funkkenner)

"#;

#[test]
fn test_from_str() {
    let config = Config::from_str(TEST_FULL_CONFIG).unwrap();
    assert_eq!(config.imap.host, "imap.gmail.com");
    assert_eq!(config.imap.port, 993);
    assert_eq!(config.imap.username, "");
    assert_eq!(config.imap.password, "");
    assert_eq!(config.interval, 25);
    assert_eq!(config.printing.min_copies, 2);
    assert_eq!(config.printing.printer, None);
    assert_eq!(config.printing.amt, 1);
}

#[test]
fn test_parse_file() {
    let config = Config::parse("examples/config.yaml").unwrap();
    assert_eq!(config.imap.host, "imap.gmail.com");
    assert_eq!(config.imap.port, 993);
    assert_eq!(config.imap.username, "");
    assert_eq!(config.imap.password, "");
    assert_eq!(config.interval, 25);
    assert_eq!(config.printing.min_copies, 2);
    assert_eq!(config.printing.printer, None);
    assert_eq!(config.printing.amt, 1);
}
