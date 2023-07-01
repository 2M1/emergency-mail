



use std::str::FromStr;


use config::logging;
use config::Config;

use log::info;

use crate::connection::imap::IMAPConnection;
use crate::models::emergency::Emergency;

mod config;
mod connection;
mod models;

fn main() {
    logging::init_logging();
    info!("starting up");
    let config_path = std::env::var("EM_CONFIG").unwrap_or("config.yaml".to_string());
    println!("config path: {}", config_path);
    let config = Config::parse(&config_path).expect("couldn't parse config");

    let mut connection = IMAPConnection::connect(&config).expect("couldn't connect to imap server");

    loop {
        let new_mails = connection.reconnecting_await_new_mail();
        for mail in new_mails {
            if mail.is_none() {
                println!("mail is none");
                continue;
            }
            println!("new mail: {:?}", mail);

            let ems = Emergency::from_str(mail.unwrap().as_str()).unwrap();
            println!("ems: {:?}", ems);
        }
    }
}
