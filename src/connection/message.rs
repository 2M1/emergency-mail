use imap::types::{Fetch, Seq};
use log::trace;

#[cfg(test)]
use crate::models::emergency::Emergency;
#[cfg(test)]
use std::str::FromStr;

use std::str;

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
    pub seq: Seq,
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
            seq: fetch.message,
            uid: fetch.uid,
            header: header,
            text: text,
        };
    }
}

// for encoding see: https://www.w3.org/Protocols/rfc1341/5_Content-Transfer-Encoding.html
pub fn mail_str_decode_unicode(str: &str) -> String {
    let mut new_str = String::with_capacity(str.len()); // replacing escape sequences can only shrink the string

    // searches for escape-sequences of the form =xx=xx where xx are hex digits and replaces them with the corresponding unicode character
    let mut prev_buffer = String::with_capacity(6);
    let mut chars = str.chars().peekable();

    while let Some(c) = chars.next() {
        if c != '=' && prev_buffer.is_empty() {
            new_str.push(c);
            continue;
        }

        if prev_buffer.len() == 1 && c == '\r' {
            // remove soft linebreaks, which are escaped with a single '='
            prev_buffer.clear();
            if chars.peek() == Some(&'\n') {
                chars.next(); // skip the \n character
            }
            trace!("found newline escape sequence");
            continue;
        }

        if c != '=' && !c.is_ascii_hexdigit() {
            new_str.push_str(&prev_buffer);
            new_str.push(c);
            prev_buffer.clear();
            continue;
        }

        prev_buffer.push(c);
        if prev_buffer.len() == 6 {
            let hex_1 = &prev_buffer[1..3];
            let hex_2 = &prev_buffer[4..6];
            let Ok(hex_1) = u8::from_str_radix(&hex_1, 16) else {
                new_str.push_str(&prev_buffer);
                prev_buffer.clear();
                continue;
            };
            let Ok(hex_2) = u8::from_str_radix(&hex_2, 16) else {
                new_str.push_str(&prev_buffer);
                prev_buffer.clear();
                continue;
            };
            let utf_codepoints = &[hex_1, hex_2].to_vec();
            let utf8 = str::from_utf8(utf_codepoints);

            if let Ok(utf8) = utf8 {
                new_str.push_str(&utf8);
            } else {
                new_str.push('�');
            }
            // if let Some(comound) = char::from_u32(utf8) {
            // }
            prev_buffer.clear();
        }
    }
    return new_str;
}

#[test]
fn test_mail_str_decode_unicode_simple() {
    // testing required unicode characters:
    assert_eq!(mail_str_decode_unicode("=C3=A4"), "ä");
    assert_eq!(mail_str_decode_unicode("=C3=A4=C3=A4"), "ää");
    assert_eq!(mail_str_decode_unicode("=C3=B6"), "ö");
    assert_eq!(mail_str_decode_unicode("=C3=BC"), "ü");
    assert_eq!(mail_str_decode_unicode("=C3=9F"), "ß");
    assert_eq!(mail_str_decode_unicode("=C3=84"), "Ä");
    assert_eq!(mail_str_decode_unicode("=C3=96"), "Ö");
    assert_eq!(mail_str_decode_unicode("=C3=9C"), "Ü");
    assert_eq!(mail_str_decode_unicode("=F4=90"), "�");

    // testing with other characters:
    assert_eq!(
        "the quick brown fox jumps over the lazy dog.",
        mail_str_decode_unicode("the quick brown fox jumps over the lazy dog.")
    );
    assert_eq!(
        "the qüick brown fox jumps over the lazy dog.",
        mail_str_decode_unicode("the q=C3=BCick brown fox jumps over the lazy dog.")
    );

    // test incomplete (should be unchanged):
    assert_eq!(
        "the q=C3=Bick brown fox jumps over the lazy dog.", // =C3=B is an incomplete escape sequence (should assume it is a normal sequence of characters)
        mail_str_decode_unicode("the q=C3=Bick brown fox jumps over the lazy dog.")
    );

    assert_eq!("testa", mail_str_decode_unicode("test=\r\na"));
}

#[test]
fn test_mail_str_decode_unicode_full() {
    // testing with full mail:
    let mail = include_str!("../../examples/emergency_bgebg_asciiescaped.txt");
    let mail = mail_str_decode_unicode(mail);
    let orig = include_str!("../../examples/emergency_bgebg.txt");

    let ems_mail = Emergency::from_str(mail.as_str()).unwrap();
    let ems_orig = Emergency::from_str(orig).unwrap();

    assert_eq!(ems_mail.town, ems_orig.town);
    assert_eq!(ems_mail.district, ems_orig.district);
    assert_eq!(ems_mail.location, ems_orig.location);
    assert_eq!(ems_mail.location_addition, ems_orig.location_addition);
    assert_eq!(ems_mail.street, ems_orig.street);
    assert_eq!(ems_mail.house_number, ems_orig.house_number);
    assert_eq!(ems_mail.object, ems_orig.object);
    assert_eq!(ems_mail.fire_department_plan, ems_orig.fire_department_plan);
    assert_eq!(ems_mail.object_part, ems_orig.object_part);
    assert_eq!(ems_mail.object_number, ems_orig.object_number);
    assert_eq!(ems_mail.emergency_type, ems_orig.emergency_type);
    assert_eq!(ems_mail.keyword, ems_orig.keyword);
    assert_eq!(ems_mail.code3, ems_orig.code3);
    assert_eq!(ems_mail.emergency_number, ems_orig.emergency_number);
    assert_eq!(ems_mail.note, ems_orig.note);
    assert_eq!(ems_mail.patient_name, ems_orig.patient_name);
    assert_eq!(ems_mail.dispatched_units, ems_orig.dispatched_units);
    // note: purposefully not comparing unit alarm times as they are hard to compare and we do not care about exact correctness.
    // given all other fields with escaped unicode characters are equal, we can assume that the unit alarm times worked as well.
    assert_eq!(ems_mail.alarm_time, ems_orig.alarm_time);
}
