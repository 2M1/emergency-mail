use std::panic::catch_unwind;
use std::str::FromStr;
use std::thread::sleep;
use std::time::Duration;

use config::logging;
use config::Config;

use log::trace;
use log::{debug, error, info};

use crate::config::config::IMAPModes::Idle;
use ctrlc;

use crate::connection::imap::IMAPConnection;
use crate::connection::message::mail_str_decode_unicode;
use crate::models::emergency::Emergency;
use crate::printing::com;
use crate::printing::print_ems::print_emergency;

mod config;
mod connection;
mod models;
mod printing;

fn poll_new_mails(
    connection: &mut IMAPConnection,
    interval: Duration,
) -> Result<Vec<Option<String>>, ()> {
    loop {
        trace!("polling tick!");
        let res = connection.load_new_mails();
        if res.is_err() || res.as_ref().is_ok_and(|v| !v.is_empty()) {
            break res; // either we have new mails or an error
        }
        sleep(interval);
    }
}

fn run_mail_loop(config: &Config) {
    let mut connection = IMAPConnection::connect(config).expect("couldn't connect to imap server");
    info!("Bereit zum Empfangen der Alarmemails.");
    loop {
        // todo: move config check out of loop
        let new_mails = if config.imap.mode.method == Idle {
            connection.reconnecting_await_new_mail()
        } else {
            poll_new_mails(&mut connection, config.interval_as_duration())
        };
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
                debug!("mail is none");
                continue;
            }

            let mail_str = mail.unwrap(); // will never panic, see check above
            let mail_str = mail_str_decode_unicode(mail_str);
            let ems = Emergency::from_str(mail_str.as_str()).unwrap();
            debug!("decoded ems id {:?}", ems.emergency_number);
            print_emergency(ems, &config);
        }
    }
}

fn main() {
    logging::init_logging();
    info!("starting up");
    let config_path = std::env::var("EM_CONFIG").unwrap_or("config.yaml".to_string());
    trace!("config path: {}", config_path);
    let config = Config::parse(&config_path);
    let Ok(config) = config else {
        error!("couldn't parse config file: {}", config.unwrap_err());
        return;
    };

    com::init().unwrap();

    ctrlc::set_handler(move || {
        info!("received SIGINT, exiting.");
        std::process::exit(0);
    })
        .expect("couldn't set SIGINT handler");

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
