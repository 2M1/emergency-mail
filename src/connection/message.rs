use imap::types::Fetch;

/// Message represents a mail message.
///
/// It is used to store the id, header and text of a mail message.
/// To be generated directly after a fetch from a Fetch struct.
/// this allows for easier testing as well as a consistent internal interface.
///
/// # example
/// ```rust
/// let fetch = session.fetch("1", "body[text]").unwrap();
/// let message = Message::from_fetch(fetch.iter().next().unwrap());
/// ```
pub struct Message {
    pub uid: Option<u32>,
    pub header: Option<Vec<u8>>,
    pub text: Option<Vec<u8>>,
}

impl Message {
    pub fn from_fetch(fetch: &Fetch) -> Self {
        // this method can unfortunately not be tested as the Fetch struct is not constructable.

        let header = if let Some(header) = fetch.header() {
            Some(header.to_vec())
        } else {
            None
        };

        let text = if let Some(text) = fetch.text() {
            Some(text.to_vec())
        } else {
            None
        };

        return Message {
            uid: fetch.uid,
            header: header,
            text: text,
        };
    }
}
