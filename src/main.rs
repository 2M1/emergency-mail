use chrono::format::format;
use config::logging;
use config::Config;

use log::info;
use no_panic::no_panic;

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

    // let mut idle = connection.idle().unwrap();
    // idle.set_keepalive(config.interval_as_duration());
    // idle.wait_keepalive().unwrap();

    // let messages = connection
    //     .fetch(
    //         "2:*",
    //         "(BODY[Header.FIELDS (Content-Type)] FLAGS UID BODY[TEXT])",
    //     )
    //     .expect("couldn't fetch message");

    // let message = messages.iter().next().unwrap();
    // let header = message.header().unwrap();
    // println!("message {:?}\n\n", message);
    // println!(
    //     "{:?}",
    //     String::from_utf8(header.to_vec()).expect("couldn't convert string")
    // );

    // connection.logout().expect("couldn't logout");
    let new = connection.load_newest();
    println!("{:?}", new);
}
