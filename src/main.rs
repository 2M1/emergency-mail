use std::panic::catch_unwind;
use std::str::FromStr;

use config::logging;
use config::Config;

use log::info;
use log::trace;

use crate::connection::imap::IMAPConnection;
use crate::connection::message::mail_str_decode_unicode;
use crate::models::emergency::Emergency;
use crate::printing::com;
use crate::printing::print_ems::print_emergency;

mod config;
mod connection;
mod models;
mod printing;

const EMERGENCY: &str = include_str!("../examples/emergency_bgebg.txt");

fn run_mail_loop(config: &Config) {
    let mut connection = IMAPConnection::connect(config).expect("couldn't connect to imap server");
    info!("ready! awaiting new mails.");
    loop {
        let new_mails = connection.reconnecting_await_new_mail();
        if new_mails.is_err() {
            connection.end();
            connection = IMAPConnection::connect(config).expect("couldn't connect to imap server");
            continue; // reconnect on error
        }
        let new_mails = new_mails.unwrap(); // will never panic, see check above

        for mail in new_mails {
            if mail.is_none() {
                trace!("mail is none");
                continue;
            }

            let mail_str = mail.unwrap();
            let mail_str = mail_str_decode_unicode(mail_str);
            let ems = Emergency::from_str(mail_str.as_str()).unwrap();
            trace!("ems: {:?}", ems);
            print_emergency(ems, &config);
        }
    }
}

fn main() {
    logging::init_logging();
    info!("starting up");
    let config_path = std::env::var("EM_CONFIG").unwrap_or("config.yaml".to_string());
    trace!("config path: {}", config_path);
    let config = Config::parse(&config_path).expect("couldn't parse config");

    com::init().unwrap();

    let ems = include_str!("../examples/emergency_many_units.txt");
    let ems = mail_str_decode_unicode(ems.to_string());
    let ems = Emergency::from_str(ems.as_str()).unwrap();
    print_emergency(ems, &config);
    loop {
        let res = catch_unwind(|| run_mail_loop(&config)); // catch panics and restart
        if res.is_err() {
            info!("caught panic, restarting");
        }
    }
}
