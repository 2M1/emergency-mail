use std::{
    borrow::BorrowMut, cell::RefCell, cmp::max, net::TcpStream, rc::Rc, sync::Arc, time::Duration,
};

use imap::{
    extensions::idle::WaitOutcome,
    types::{Fetch, Mailbox, UnsolicitedResponse},
    Session,
};
use log::{error, info, warn};
use native_tls::TlsStream;
use no_panic::no_panic;

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
        let client = imap::ClientBuilder::new(imap_cfg.host, imap_cfg.port)
            .native_tls()
            .map_err(|e| format!("couldn't create imap client: {}", e))?;
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
        let mut idle = self.session.idle();
        idle.timeout(self.idle_interval);

        let err = idle.wait_while(|response| {
            println!("response: {:?}", response);

            match response {
                UnsolicitedResponse::Exists(exists) | UnsolicitedResponse::Recent(exists) => {
                    info!("{} new max/new mails", exists);
                    // let mails = this.load_newest();
                    // f(mails);
                }
                _ => {
                    // TODO: check if a new mail is available
                }
            }

            true
        });

        match err {
            Ok(_) => return IMAPIdleError::ConnectionError, // idle returned false, due to a timeout
            Err(e) => {
                error!("idle error: {}", e);
                return IMAPIdleError::InitialisationError;
            }
        }
    }

    pub fn reconnecting_on_new_mail(&mut self, f: &mut dyn FnMut(Vec<Option<String>>)) -> ! {
        let mut init_err_count = 0;
        loop {
            match self.on_new_mail(f) {
                IMAPIdleError::InitialisationError => {
                    init_err_count += 1;
                    if init_err_count > 5 {
                        error!("too many initialisation errors");
                        // TODO: decide on strategy
                        // currently: reconnect immediately
                    }
                    info!("reconnecting");
                }

                IMAPIdleError::ConnectionError => {
                    info!("reconnecting");
                    init_err_count = 0;
                }
            }
        }
    }

    pub fn end(&mut self) {
        self.session.logout().unwrap_or_else(|e| {
            warn!("couldn't logout: {}", e);
        });
    }
}
