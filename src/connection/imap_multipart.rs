use super::message::Message;

use log::{error, warn};

/// Extracts the plain text from a multipart mail.
///
/// # pre-conditions
///
/// The mail is expected to be a multipart mail, starting with the separator. (which in itself starts
/// with "--" and ends with "\r\n")
/// The separator is always followed by the part specific headers, which are followed by the body of the part.
///
/// # arguments
///
/// * `content` - the content of the mail, starting with the separator
///
/// # return value
///
/// The plain text of the mail is returned, if it exists and the mail was multipart.
///
/// For mails that do not contain a multipart body, None is returned - even if the mail itself is valid plain text content.
/// $\implies$ Before calling this function, the mail headers (not within the body) should be checked for content-type multipart.
///
pub fn extract_multipart_plain_text(content: &str) -> Option<String> {
    // @pre: content is a multipart mail, starting with the separator

    let Some(sepereator_end) = content.find("\r\n") else {
        error!("couldn't find separator end");
        return None;
    };
    let separator = content[..sepereator_end].to_string();

    let parts = content.split(&separator);
    for part in parts {
        let header_end = part.find("\r\n\r\n").unwrap_or_default();
        let headers = &part[..header_end].to_string();
        if headers.is_empty() {
            continue;
        }
        let headers = headers.split("\r\n");
        let mut is_plain_text = false;
        for header in headers {
            if header.starts_with("Content-Type: text/plain") {
                is_plain_text = true;
                break;
            }
        }
        if !is_plain_text {
            continue;
        }

        let body = &part[header_end + 4..];
        return Some(body.to_string());
    }

    return None;
}

fn headers_get_content_type(headers: Vec<u8>) -> Option<String> {
    const HEADER_CONTENT_TYPE: &str = "Content-Type: ";
    let Ok(headers) = String::from_utf8(headers.to_vec()).map_err(|_| {
        warn!("couldn't convert headers to string");
    }) else {
        return None;
    };

    let headers = headers.split("\r\n");
    for header in headers {
        if header.starts_with(HEADER_CONTENT_TYPE) {
            let content_type = header[HEADER_CONTENT_TYPE.len()..].to_string();
            return Some(content_type);
        }
    }

    return None;
}

pub fn get_message_body(message: Message) -> Option<String> {
    let Some(body) = message.text else {
        error!(
            "couldn't get body of mail {}",
            message.uid.unwrap_or_default()
        );
        return None;
    };

    let content = String::from_utf8(body.to_vec()).map_err(|_| {
        error!(
            "couldn't convert mail {} to string.",
            message.uid.unwrap_or_default()
        );
    });
    let Ok(content) = content else {
        return None;
    };

    if let Some(headers) = message.header {
        let content_type = headers_get_content_type(headers);
        if let Some(content_type) = content_type {
            if content_type.contains("multipart/alternative") {
                return extract_multipart_plain_text(&content);
            }
            if content_type.trim() == "text/plain" {
                // plain text mail
                return Some(content);
            }
        }
    }

    // check if the mail is multipart, even if no headers were part of the response
    // (due to the imap fetch request not including the content-type header o.s.)
    if content.starts_with("--") {
        // multipart mail
        // this sequence from -- to \r\n is used to separate the different parts of the mail
        // possible parts are: text/plain, text/html, etc.
        // we only want the text/plain part
        return extract_multipart_plain_text(&content);
    }

    return Some(content);
}

#[cfg(test)]
pub mod test {

    use super::extract_multipart_plain_text;

    pub const MULTIPART_BODY: &str = "--fcd0a2e3-f220-407c-96ea-a69339f943bc-1\r\nContent-Type: text/plain; charset=\"utf-8\"\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\n~~Ort~~Brandenburg an der Havel~~\r\n\r\n\r\n~~Ortsteil~~G=C3=B6ttin/BRB~~\r\n\r\n\r\n\r\n~~Ortslage~~G=C3=B6risgr=C3=A4ben~~\r\n\r\n\r\n\r\n~~Strasse~~G=C3=B6risgr=C3=A4ben~~\r\n\r\n\r\n\r\n~~Hausnummer~~22~~\r\n\r\n\r\n\r\n~~Objekt~~~~\r\n\r\n\r\n\r\n~~FWPlan~~~~\r\n\r\n\r\n\r\n~~Objektteil~~~~\r\n\r\n\r\n\r\n~~Objektnummer~~-1~~\r\n\r\n\r\n\r\n~~Einsatzart~~Hilfeleistungseinsatz~~\r\n\r\n\r\n\r\n~~Alarmgrund~~H:Natur~~\r\n\r\n\r\n\r\n~~Sondersignal~~ohne Sondersignal~~\r\n\r\n\r\n\r\n~~Einsatznummer~~322088295~~\r\n\r\n\r\n\r\n~~Besonderheiten~~TESTETESTTESTE~~\r\n\r\n\r\n\r\n~~Name~~,~~\r\n\r\n\r\n\r\n~~EMListe~~FL BRB 01/16-21, RLS BRB DGL 2~~\r\n\r\n\r\n\r\n~~Status~~Fahrzeug~~Zuget~~Alarm~~Ausger=C3=BCckt~~\r\n\r\n\r\n\r\n~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8~~FL BRB 01/16-21~~08:21~~=\r\n~~\r\n\r\n\r\n\r\n~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8~~FL BRB 01/16-21~~08:21~~=\r\n~~\r\n\r\n\r\n\r\n~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8~~RLS BRB DGL 2~~08:23~~~~\r\n\r\n\r\n\r\n~~WGS84_X~~52.33823333~~\r\n\r\n\r\n\r\n~~WGS84_Y~~12.48626667~~\r\n\r\n\r\n\r\n~~Koord_EPSG_25833~~12.48626667~~52.33823333~~\r\n\r\n\r\n\r\n~~Koord_EPSG_4326~~E1248630~~N5233820~~~~Einsatzortzusatz~~~~\r\n\r\n\r\n\r\n~~Alarmzeit~~29.09.22&08:23~~\r\n--fcd0a2e3-f220-407c-96ea-a69339f943bc-1\r\nContent-Type: text/html; charset=\"utf-8\"\r\nContent-Transfer-Encoding: quoted-printable\r\n\r\n<!DOCTYPE html><html><head><meta http-equiv=3D\"Content-Type\" content=3D\"t=\r\next/html; charset=3Dutf-8\"></head><body><div style><div style><div style>=\r\n<div style><div style>~~Ort~~Brandenburg an der Havel~~<u style></u></div=\r\n><div style><div dir=3D\"ltr\" style><div style><div style><div link=3D\"#05=\r\n63C1\" vlink=3D\"#954F72\" style=3D\"overflow-wrap: break-word;\" lang=3D\"DE\">=\r\n<div style><p style>~~Ortsteil~~G=C3=B6ttin/BRB~~<u style></u></p><p styl=\r\ne>~~Ortslage~~G=C3=B6risgr=C3=A4ben~~<u style></u></p><p style>~~Strasse~=\r\n~G=C3=B6risgr=C3=A4ben~~<u style></u></p><p style>~~Hausnummer~~22~~<u st=\r\nyle></u></p><p style>~~Objekt~~~~<u style></u></p><p style>~~FWPlan~~~~<u=\r\n style></u></p><p style>~~Objektteil~~~~<u style></u></p><p style>~~Objek=\r\ntnummer~~-1~~<u style></u></p><p style>~~Einsatzart~~Hilfeleistungseinsat=\r\nz~~<u style></u></p><p style>~~Alarmgrund~~H:Natur~~<u style></u></p><p s=\r\ntyle>~~Sondersignal~~ohne Sondersignal~~<u style></u></p><p style>~~Einsa=\r\ntznummer~~322088295~~<u style></u></p><p style>~~Besonderheiten~~TESTETES=\r\nTTESTE~~<u style></u></p><p style>~~Name~~,~~<u style></u></p><p style>~~=\r\nEMListe~~FL BRB 01/16-21, RLS BRB DGL 2~~<u style></u></p><p style>~~Stat=\r\nus~~Fahrzeug~~Zuget~~Alarm~~Ausger=C3=BCckt~~<u style></u></p><p style>~~=\r\nALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8~~FL BRB 01/16-21~~08:21~~~~=\r\n<u style></u></p><p style>~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8=\r\n~~FL BRB 01/16-21~~08:21~~~~<u style></u></p><p style>~~ALARM~~unbekannt#=\r\n~~BRB FW Brandenburg 1=C3=B8~~RLS BRB DGL 2~~08:23~~~~<u style></u></p><p=\r\n style>~~WGS84_X~~52.33823333~~<u style></u></p><p style>~~WGS84_Y~~12.48=\r\n626667~~<u style></u></p><p style>~~Koord_EPSG_25833~~12.48626667~~52.338=\r\n23333~~<u style></u></p><p style>~~Koord_EPSG_4326~~E1248630~~N5233820~~~=\r\n~Einsatzortzusatz~~~~<u style></u></p><p style>~~Alarmzeit~~29.09.22&amp;=\r\n08:23~~<u style></u></p></div></div></div></div></div></div><div style><b=\r\nr></div></div></div></div></div></body></html>\r\n--fcd0a2e3-f220-407c-96ea-a69339f943bc-1--\r\n";
    pub const MULTIPART_BODY_PLAIN_TEXT: &str = "~~Ort~~Brandenburg an der Havel~~\r\n\r\n\r\n~~Ortsteil~~G=C3=B6ttin/BRB~~\r\n\r\n\r\n\r\n~~Ortslage~~G=C3=B6risgr=C3=A4ben~~\r\n\r\n\r\n\r\n~~Strasse~~G=C3=B6risgr=C3=A4ben~~\r\n\r\n\r\n\r\n~~Hausnummer~~22~~\r\n\r\n\r\n\r\n~~Objekt~~~~\r\n\r\n\r\n\r\n~~FWPlan~~~~\r\n\r\n\r\n\r\n~~Objektteil~~~~\r\n\r\n\r\n\r\n~~Objektnummer~~-1~~\r\n\r\n\r\n\r\n~~Einsatzart~~Hilfeleistungseinsatz~~\r\n\r\n\r\n\r\n~~Alarmgrund~~H:Natur~~\r\n\r\n\r\n\r\n~~Sondersignal~~ohne Sondersignal~~\r\n\r\n\r\n\r\n~~Einsatznummer~~322088295~~\r\n\r\n\r\n\r\n~~Besonderheiten~~TESTETESTTESTE~~\r\n\r\n\r\n\r\n~~Name~~,~~\r\n\r\n\r\n\r\n~~EMListe~~FL BRB 01/16-21, RLS BRB DGL 2~~\r\n\r\n\r\n\r\n~~Status~~Fahrzeug~~Zuget~~Alarm~~Ausger=C3=BCckt~~\r\n\r\n\r\n\r\n~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8~~FL BRB 01/16-21~~08:21~~=\r\n~~\r\n\r\n\r\n\r\n~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8~~FL BRB 01/16-21~~08:21~~=\r\n~~\r\n\r\n\r\n\r\n~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8~~RLS BRB DGL 2~~08:23~~~~\r\n\r\n\r\n\r\n~~WGS84_X~~52.33823333~~\r\n\r\n\r\n\r\n~~WGS84_Y~~12.48626667~~\r\n\r\n\r\n\r\n~~Koord_EPSG_25833~~12.48626667~~52.33823333~~\r\n\r\n\r\n\r\n~~Koord_EPSG_4326~~E1248630~~N5233820~~~~Einsatzortzusatz~~~~\r\n\r\n\r\n\r\n~~Alarmzeit~~29.09.22&08:23~~\r\n";

    #[test]
    pub fn test_extract_multipart_plain_text_from_multipart() {
        let plaintext = extract_multipart_plain_text(MULTIPART_BODY);
        assert!(plaintext.is_some());
        assert_eq!(plaintext.unwrap(), MULTIPART_BODY_PLAIN_TEXT);
    }

    #[test]
    pub fn test_extract_multipart_plain_text_from_non_multipart() {
        let plaintext = extract_multipart_plain_text(MULTIPART_BODY_PLAIN_TEXT);
        assert!(plaintext.is_none());
    }

    #[test]
    pub fn test_extract_multipart_plain_text_from_empty() {
        let plaintext = extract_multipart_plain_text("");
        assert!(plaintext.is_none());
    }
}
