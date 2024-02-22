use imap::types::{Fetch, Seq};

#[cfg(test)]
use crate::models::emergency::Emergency;
#[cfg(test)]
use std::str::FromStr;

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

pub fn mail_str_decode_unicode(str: String) -> String {
    let mut new_str = String::with_capacity(str.len()); // replacing escape sequences can only shrink the string

    // searches for escapesequences of the form =xx=xx where xx are hex digits and replaces them with the corresponding unicode character
    let mut prev_buffer = String::with_capacity(6);
    for c in str.chars() {
        if c != '=' && prev_buffer.is_empty() {
            new_str.push(c);
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
            let hex_1 = u8::from_str_radix(&hex_1, 16).unwrap();
            let hex_2 = u8::from_str_radix(&hex_2, 16).unwrap();
            let utf8 = String::from_utf8([hex_1, hex_2].to_vec()).unwrap();

            // if let Some(comound) = char::from_u32(utf8) {
            new_str.push_str(utf8.as_str());
            // }
            prev_buffer.clear();
        }
    }
    return new_str;
}

#[test]
fn test_mail_str_decode_unicode_simple() {
    // testing required unicode characters:
    assert_eq!(
        mail_str_decode_unicode("=C3=A4".to_string()),
        "ä".to_string()
    );
    assert_eq!(
        mail_str_decode_unicode("=C3=A4=C3=A4".to_string()),
        "ää".to_string()
    );
    assert_eq!(
        mail_str_decode_unicode("=C3=B6".to_string()),
        "ö".to_string()
    );
    assert_eq!(
        mail_str_decode_unicode("=C3=BC".to_string()),
        "ü".to_string()
    );
    assert_eq!(
        mail_str_decode_unicode("=C3=9F".to_string()),
        "ß".to_string()
    );
    assert_eq!(
        mail_str_decode_unicode("=C3=84".to_string()),
        "Ä".to_string()
    );
    assert_eq!(
        mail_str_decode_unicode("=C3=96".to_string()),
        "Ö".to_string()
    );
    assert_eq!(
        mail_str_decode_unicode("=C3=9C".to_string()),
        "Ü".to_string()
    );

    // testing with other characters:
    assert_eq!(
        "the quick brown fox jumps over the lazy dog.".to_string(),
        mail_str_decode_unicode("the quick brown fox jumps over the lazy dog.".to_string())
    );
    assert_eq!(
        "the qüick brown fox jumps over the lazy dog.".to_string(),
        mail_str_decode_unicode("the q=C3=BCick brown fox jumps over the lazy dog.".to_string())
    );

    // test incomplete (should be unchanged):
    assert_eq!(
        "the q=C3=Bick brown fox jumps over the lazy dog.".to_string(), // =C3=B is an incomplete escape sequence (should assume it is a normal sequence of characters)
        mail_str_decode_unicode("the q=C3=Bick brown fox jumps over the lazy dog.".to_string())
    );
}

#[test]
fn test_mail_str_decode_unicode_full() {
    // testing with full mail:
    let mail = include_str!("../../examples/emergency_bgebg_asciiescaped.txt");
    let mail = mail_str_decode_unicode(mail.to_string());
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
