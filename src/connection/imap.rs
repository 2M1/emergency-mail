use std::net::TcpStream;

use imap::Session;
use native_tls::{TlsConnector, TlsStream};

use crate::config::Config;




pub fn connect(config: &Config) -> Result<Session<TlsStream<TcpStream>>, String> {

    let smtp_cfg = config.imap.clone();
    let tls = TlsConnector::builder().build()
        .map_err(|e| format!("couldn't create TLS connector: {}", e))?;
    let client = imap::connect((smtp_cfg.host, smtp_cfg.port), &config.imap.host, &tls)
        .map_err( |e| format!("couldn't connect to imap server: {}", e))?;
    let session =  client.login(smtp_cfg.username, smtp_cfg.password)
        .map_err(|e| format!("couldn't login to imap server: {}", e.0))?;
    return Ok(session);
}