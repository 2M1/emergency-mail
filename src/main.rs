use std::panic::catch_unwind;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use config::logging;
use config::Config;

use log::debug;
use log::error;
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
    info!("Bereit zum Empfangen der Alarmemails.");
    loop {
        let new_mails = connection.reconnecting_await_new_mail();
        if new_mails.is_err() {
            connection.end();
            info!("reconnecting to imap server");
            let reconnection = IMAPConnection::connect(config);
            if reconnection.is_err() {
                error!("couldn't reconnect to imap server");
                return;
            }
            info!("Bereit zum Empfangen der Alarmemails.");
            connection = reconnection.unwrap(); // will never panic, see check above
            continue;
        }
        let new_mails = new_mails.unwrap(); // will never panic, see check above

        for mail in new_mails {
            if mail.is_none() {
                trace!("mail is none");
                continue;
            }

            let mail_str = mail.unwrap(); // will never panic, see check above
            let mail_str = mail_str_decode_unicode(mail_str);
            let ems = Emergency::from_str(mail_str.as_str()).unwrap();
            debug!("ems: {:?}", ems);
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

    /* let ems = include_str!("../examples/emergency_many_units.txt");
    let ems = mail_str_decode_unicode(ems.to_string());
    let ems = Emergency::from_str(ems.as_str()).unwrap();
    print_emergency(ems, &config);*/
    let mut is_first = true;
    loop {
        let res = catch_unwind(|| run_mail_loop(&config)); // catch panics and restart
        if res.is_err() {
            info!("caught panic, restarting");
            trace!("panic: {:?}", res);
            if is_first {
                is_first = false;
                continue;
            }
            // wait for 10 seconds before restarting
            // this is to slow a panic loop down, so that the log messages can be read
            sleep(Duration::from_secs(10));
        } else {
            is_first = true;
        }
    }
}
