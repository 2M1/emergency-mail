
use config::logging;
use config::Config;

use log::info;


use crate::connection::imap::IMAPConnection;

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

    connection.reconnecting_on_new_mail(&mut |mails| {
        for mail in mails {
            println!("new mail: {:?}", mail);
        }
    });
}
