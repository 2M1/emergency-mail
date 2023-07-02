use std::str::FromStr;

use config::logging;
use config::Config;

use log::info;
use log::trace;

use crate::connection::imap::IMAPConnection;
use crate::models::emergency::Emergency;
use crate::printing::com;
use crate::printing::print_xps::print_test;
use crate::printing::xps_document::XPSSingleDocument;

mod config;
mod connection;
mod models;
mod printing;

fn main() {
    logging::init_logging();
    info!("starting up");
    let config_path = std::env::var("EM_CONFIG").unwrap_or("config.yaml".to_string());
    trace!("config path: {}", config_path);
    let config = Config::parse(&config_path).expect("couldn't parse config");

    com::init().unwrap();

    let mut doc = XPSSingleDocument::new().unwrap();
    doc.newPage();
    doc.addText(1, "Hello world".to_string(), 10.0, 10.0);
    doc.newPage();
    doc.safe();
    print_test(doc);

    // let mut connection = IMAPConnection::connect(&config).expect("couldn't connect to imap server");
    // trace!("ready! awaiting new mails.");
    // loop {
    //     let new_mails = connection.reconnecting_await_new_mail();
    //     for mail in new_mails {
    //         if mail.is_none() {
    //             trace!("mail is none");
    //             continue;
    //         }

    //         let ems = Emergency::from_str(mail.unwrap().as_str()).unwrap();
    //         trace!("ems: {:?}", ems);
    //     }
    // }
}
