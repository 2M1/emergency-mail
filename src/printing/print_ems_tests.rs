use std::{env, str::FromStr};

use crate::{config::Config, models::emergency::Emergency, printing::print_ems::count_copies};

const EMS_FULL: &'static str = include_str!("../../examples/emergency_bgebg.txt");
const EMS_NONE: &'static str = include_str!("../../examples/emergency_obj.txt");
const EMS_ONE: &'static str = include_str!("../../examples/emergency_simple.txt");

#[test]
fn test_count_copies_many() {
    // required for config parsing
    env::set_var("EM_IMAP_HOST", "host");
    env::set_var("EM_IMAP_USERNAME", "user");
    env::set_var("EM_IMAP_PASSWORD", "pass");

    let configFull = Config::parse("examples/config_full.yaml").unwrap(); // min_copies = 2, max_copies = 5
    let configMin = Config::parse("examples/config.yaml").unwrap(); // min_copies = 1, max_copies = None
    let ems = Emergency::from_str(EMS_FULL).unwrap();
    assert_eq!(count_copies(&ems, &configFull), 5);
    assert_eq!(count_copies(&ems, &configMin), 12);
}

#[test]
fn test_count_copies_none() {
    // required for config parsing
    env::set_var("EM_IMAP_HOST", "host");
    env::set_var("EM_IMAP_USERNAME", "user");
    env::set_var("EM_IMAP_PASSWORD", "pass");

    let configFull = Config::parse("examples/config_full.yaml").unwrap(); // min_copies = 1, max_copies = 5
    let configMin = Config::parse("examples/config.yaml").unwrap(); // min_copies = 1, max_copies = None
    let ems = Emergency::from_str(EMS_NONE).unwrap();
    assert_eq!(count_copies(&ems, &configFull), 2); // min_copies = 1
    assert_eq!(count_copies(&ems, &configMin), 1);
}

#[test]
fn test_count_copies_one() {
    // required for config parsing
    env::set_var("EM_IMAP_HOST", "host");
    env::set_var("EM_IMAP_USERNAME", "user");
    env::set_var("EM_IMAP_PASSWORD", "pass");

    let configFull = Config::parse("examples/config_full.yaml").unwrap(); // min_copies = 1, max_copies = 5
    let configMin = Config::parse("examples/config.yaml").unwrap(); // min_copies = 1, max_copies = None
    let ems = Emergency::from_str(EMS_ONE).unwrap();
    assert_eq!(count_copies(&ems, &configFull), 2); // min_copies = 2
    assert_eq!(count_copies(&ems, &configMin), 1);
}
