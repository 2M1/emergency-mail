use std::{cmp::max, net::TcpStream, sync::Arc, time::Duration};

use imap::{
    types::{Fetch, Mailbox},
    Session,
};
use log::{error, warn};
use native_tls::{TlsConnector, TlsStream};

use crate::config::Config;

const MAILBOX_INBOX: &str = "INBOX";

pub enum IMAPIdleError {
    InitialisationError,
    ConnectionError,
}

pub struct IMAPConnection {
    session: Session<TlsStream<TcpStream>>,
    inbox: Mailbox,
    idle_interval: Duration,
}

impl IMAPConnection {
    pub fn connect(config: &Config) -> Result<Self, String> {
        let imap_cfg = config.imap.clone();
        let tls = TlsConnector::builder()
            .build()
            .map_err(|e| format!("couldn't create TLS connector: {}", e))?;
        let client = imap::connect((imap_cfg.host, imap_cfg.port), &config.imap.host, &tls)
            .map_err(|e| format!("couldn't connect to imap server: {}", e))?;
        let mut session = client
            .login(imap_cfg.username, imap_cfg.password)
            .map_err(|e| format!("couldn't login to imap server: {}", e.0))?;

        let inbox = session
            .select(MAILBOX_INBOX)
            .map_err(|e| format!("couldn't select mailbox {}: {}", MAILBOX_INBOX, e))?;

        return Ok(Self {
            session: session,
            inbox: inbox,
            idle_interval: config.interval_as_duration(),
        });
    }

    fn message_body(message: &Fetch) -> Option<String> {
        let Some(body) = message.body() else {
            error!(
                "couldn't get body of mail {}",
                message.uid.unwrap_or_default()
            );
            return None;
        };

        return String::from_utf8(body.to_vec())
            .map_err(|_| {
                error!(
                    "couldn't convert mail {} to string.",
                    message.uid.unwrap_or_default()
                );
            })
            .ok();
    }

    pub fn load_newest(&mut self) -> Vec<Option<String>> {
        return self
            .session
            .fetch(
                format!("{}:*", self.inbox.exists),
                "(BODY[Header.FIELDS (Content-Type)] FLAGS UID BODY[TEXT])",
            )
            .expect("couldn't fetch message")
            .iter()
            .map(|m| {
                self.inbox.exists = max(self.inbox.exists, m.uid.unwrap_or_default() + 1);
                return m;
            })
            .map(Self::message_body)
            .collect();
    }

    pub fn on_new_mail(&mut self, f: &mut dyn FnMut(Vec<Option<String>>)) -> IMAPIdleError {
        let Ok(mut idle) = self.session.idle() else {
            error!("couldn't start idle");
            return IMAPIdleError::InitialisationError;
        };

        idle.set_keepalive(self.idle_interval);
        loop {
            let Ok(_) = idle.wait_keepalive() else {
                error!("idle connection lost!");
                return IMAPIdleError::ConnectionError;
            };
            // f(self.load_newest());
        }
    }

    pub fn end(&mut self) {
        self.session.logout().unwrap_or_else(|e| {
            warn!("couldn't logout: {}", e);
        });
    }
}
