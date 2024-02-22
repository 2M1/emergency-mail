use std::{cmp::max, net::TcpStream, time::Duration};

use imap::{
    types::{Mailbox, UnsolicitedResponse},
    ImapConnection, Session,
};

use log::{debug, error, info, trace, warn};
use native_tls::TlsStream;

use crate::config::Config;

use super::imap_multipart::get_message_body;
use super::message::Message;

const MAILBOX_INBOX: &str = "INBOX";

/// Possible errors that can occur when using the IMAP IDLE command.
pub enum IMAPIdleError {
    InitialisationError,
    ConnectionError,
}

/// Represents a connection to an IMAP server.
pub struct IMAPConnection {
    session: Session<Box<dyn ImapConnection>>,
    inbox: Mailbox,
    idle_interval: Duration,
}

impl IMAPConnection {
    /// Creates a new IMAPConnection by connecting and authenticating to the server specified in the config.
    ///
    /// # description
    /// Connects to the IMAPServer in config, using native OpenSSL (TLS).
    /// It then logs in to the server using the username and password specified in the config and selects the INBOX mailbox.
    pub fn connect(config: &Config) -> Result<Self, String> {
        let imap_cfg = &config.imap;
        let client = imap::ClientBuilder::new(&imap_cfg.host, imap_cfg.port)
            .connect()
            .map_err(|e| format!("couldn't create imap client: {}", e))?;
        let mut session = client
            .login(&imap_cfg.username, &imap_cfg.password)
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

    /// loads the newest mails in the inbox from the server
    ///
    /// # description
    /// fetches all mails with an uid greater than the current max uid in the inbox.
    /// the uid value is transmitted when selecting the inbox in the connect method.
    /// the uid is then updated to the maximum uid of the fetched mails.
    /// Therefore consecutive calls to this method will only fetch new mails and *should not* fetch the same message twice.
    ///
    #[deprecated(
        note = "fails with an empty vec, use load_new_mails instead to get a result return"
    )]
    pub fn load_newest(&mut self) -> Vec<Option<String>> {
        return self.load_since(self.inbox.exists + 1).unwrap_or_else(|e| {
            error!("couldn't load newest mails: {:?}", e);
            Vec::new()
        });
    }

    /// loads the newest mails in the inbox from the server
    ///
    /// # description
    /// fetches all mails with an uid greater than the current max uid in the inbox.
    /// the uid value is transmitted when selecting the inbox in the connect method.
    /// the uid is then updated to the maximum uid of the fetched mails.
    /// Therefore consecutive calls to this method will only fetch new mails and *should not* fetch the same message twice.
    ///
    pub fn load_new_mails(&mut self) -> Result<Vec<Option<String>>, ()> {
        // return self.load_since(self.inbox.exists + 1);
        let messages = self
            .session
            .uid_fetch("*", "(BODY[Header.FIELDS (Content-Type)] FLAGS UID)");
        if let Err(e) = messages {
            error!("failed to fetch newest message: {}", e);
            return Err(());
        }
        let messages = messages.unwrap();

        let exists_curr = self.inbox.exists; // must be saved before filter, since it would otherwise be borrowed twice (once mutable in map)
        if messages
            .iter()
            .map(Message::from_fetch)
            .filter(|m| m.header.is_some())
            .any(|m| m.seq > exists_curr)
        {
            // there is a message with a higher sequence number than the current max seq nr.
            // unfortunately uid_fetch only fetches the most recent message, so we have to fetch all since last again

            // note: reduces the risk of race conditions with new mails arriving while fetching, because
            // load since uses an open interval, so it will always fetch all new mails.
            return self.load_since(exists_curr + 1);
        }

        return Ok(vec![]);
    }

    /// loads the newest mails in the inbox from the server
    ///
    /// # description
    /// fetches all mails with a uid greater than the current max uid in the inbox.
    /// the uid value is transmitted when selecting the inbox in the connect method.
    /// the uid is then updated to the maximum uid of the fetched mails.
    /// Therefore consecutive calls to this method will only fetch new mails and *should not* fetch the same message twice.
    ///
    pub fn load_since(&mut self, min_id: u32) -> Result<Vec<Option<String>>, ()> {
        let set = format!("{}:*", min_id);
        debug!("fetching mails with set {}", set);
        let fetch_res = self.session.fetch(
            set,
            "(BODY[Header.FIELDS (Content-Type)] FLAGS UID BODY[TEXT])",
        );
        if let Err(e) = fetch_res {
            error!("couldn't fetch new mails: {}", e);
            return Err(());
        }

        let messages = fetch_res.unwrap(); // will never panic, see check above
        return Ok(messages
            .iter()
            .map(Message::from_fetch)
            .filter(|m| m.text.is_some())
            .map(|m| {
                trace!("fetched message: {:?}", m.uid);
                // set the maximum uid currently seen.

                self.inbox.exists = max(self.inbox.exists, m.seq);

                return m;
            })
            .map(get_message_body)
            .collect());
    }

    pub fn most_current_id(&self) -> u32 {
        return self.inbox.exists;
    }

    /// waits for new mails to arrive in the inbox using the IMAP IDLE command.
    ///
    /// # description
    /// the IMAP IDLE command is used to wait for new mails to arrive in the inbox.
    /// the connection is automatically re-established on the RFC specified timeout period (slow poll).
    /// but not on other errors.
    pub fn await_new_mail(&mut self) -> Result<u32, IMAPIdleError> {
        let mut idle = self.session.idle();
        idle.timeout(self.idle_interval);
        let mut new_max = self.inbox.exists;

        let err = idle.wait_while(|response| {
            return match response {
                UnsolicitedResponse::Exists(exists) => {
                    new_max = max(new_max, exists);
                    false
                }
                UnsolicitedResponse::Recent(num) => {
                    info!("{} recent mails", num);
                    true
                }
                _ => {
                    // TODO: check if a new mail is available
                    trace!("unrecognised unsolicited response: {:?}", response);
                    true
                }
            };
        });

        return match err {
            Ok(_) => Ok(new_max), // idle returned false, due to a timeout
            Err(e) => {
                error!("idle error: {}", e);
                Err(IMAPIdleError::InitialisationError)
            }
        };
    }

    /// waits for new mails to arrive in the inbox using the IMAP IDLE command. Reconnects on error.
    ///
    /// # description
    /// similar to await_new_mail, but reconnects on errors.
    /// The reconnect strategy is currently to reconnect immediately. In the future this might be changed
    /// to wait for a dynamically calculated time period, based on the number of errors, before retrying.
    ///
    pub fn reconnecting_await_new_mail(&mut self) -> Result<Vec<Option<String>>, ()> {
        const MAX_RECONNECTION_ATTEMPTS: u8 = 3; // try to reconnect, but fail rather fast
        let mut init_err_count = 0;
        loop {
            let result = self.await_new_mail();
            if let Ok(exists) = result {
                info!("new mail nr: {}", exists);
                let newest = self.load_newest();
                return Ok(newest);
            } else if let Err(e) = result {
                match e {
                    IMAPIdleError::InitialisationError => {
                        init_err_count += 1;
                        if init_err_count > MAX_RECONNECTION_ATTEMPTS {
                            error!("too many initialisation errors");
                            // strategy: error out and signal to the main process to reestablish
                            // the connection from the get go (not only the idle request).
                            return Err(());
                        }
                        info!("resetting idle connection");
                    }

                    IMAPIdleError::ConnectionError => {
                        info!("resetting idle connection");
                        init_err_count = 0;
                    }
                }
            }
        }
    }

    /// ends the session by logging out
    pub fn end(&mut self) {
        self.session.logout().unwrap_or_else(|e| {
            warn!("couldn't logout: {}", e);
        });
    }
}
