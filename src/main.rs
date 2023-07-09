use std::str::FromStr;

use config::logging;
use config::Config;

use log::info;
use log::trace;

use crate::models::emergency::Emergency;
use crate::printing::com;
use crate::printing::print_ems::print_emergency;



mod config;
mod connection;
mod models;
mod printing;

const EMERGENCY: &str = r#"~~Ort~~Brandenburg an der Havel~~

    ~~Ortsteil~~Göttin/BRB~~
    
    ~~Ortslage~~Görisgräben~~
    
    ~~Strasse~~Görisgräben~~
    
    ~~Hausnummer~~22~~
    
    ~~Objekt~~~~
    
    ~~FWPlan~~~~
    
    ~~Objektteil~~~~
    
    ~~Objektnummer~~-1~~
    
    ~~Einsatzart~~Hilfeleistungseinsatz~~
    
    ~~Alarmgrund~~H:Natur~~
    
    ~~Sondersignal~~ohne Sondersignal~~
    
    ~~Einsatznummer~~322088295~~
    
    ~~Besonderheiten~~TESTETESTTESTE~~
    
    ~~Name~~,~~
    
    ~~EMListe~~FL BRB 01/16-21, RLS BRB DGL 2~~
    
    ~~Status~~Fahrzeug~~Zuget~~Alarm~~Ausgerückt~~
    
    ~~ALARM~~unbekannt#~~BRB FW Brandenburg 1ø~~FL BRB 01/16-21~~08:21~~~~
    
    ~~ALARM~~unbekannt#~~BRB FW Brandenburg 1ø~~FL BRB 01/16-21~~08:21~~~~
    
    ~~ALARM~~unbekannt#~~BRB FW Brandenburg 1ø~~RLS BRB DGL 2~~08:23~~~~
    
    ~~WGS84_X~~52.33823333~~
    
    ~~WGS84_Y~~12.48626667~~
    
    ~~Koord_EPSG_25833~~12.48626667~~52.33823333~~
    
    ~~Koord_EPSG_4326~~E1248630~~N5233820~~~~Einsatzortzusatz~~~~
    
    ~~Alarmzeit~~29.09.22&08:23~~"#;

fn main() {
    logging::init_logging();
    info!("starting up");
    let config_path = std::env::var("EM_CONFIG").unwrap_or("config.yaml".to_string());
    trace!("config path: {}", config_path);
    let _config = Config::parse(&config_path).expect("couldn't parse config");

    com::init().unwrap();

    let ems = Emergency::from_str(EMERGENCY).unwrap();
    print_emergency(ems);
    // print_test(doc);

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
