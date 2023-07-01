use crate::connection::{
    imap_multipart::{get_message_body, test::MULTIPART_BODY, test::MULTIPART_BODY_PLAIN_TEXT},
    message::Message,
};

const HEADER_CONTENT_TYPE: &str = "Content-Type: text/plain\r\n";
const HEADER_CONTENT_TYPE_MULTIPART: &str =
    "Content-Type: multipart/alternative; boundary=--fcd0a2e3-f220-407c-96ea-a69339f943bc-1\r\n";
const BODY_PLAIN_TEXT: &str = "~~Ort~~Brandenburg an der Havel~~\r\n\r\n\r\n~~Ortsteil~~G=C3=B6ttin/BRB~~\r\n\r\n\r\n\r\n~~Ortslage~~G=C3=B6risgr=C3=A4ben~~\r\n\r\n\r\n\r\n~~Strasse~~G=C3=B6risgr=C3=A4ben~~\r\n\r\n\r\n\r\n~~Hausnummer~~22~~\r\n\r\n\r\n\r\n~~Objekt~~~~\r\n\r\n\r\n\r\n~~FWPlan~~~~\r\n\r\n\r\n\r\n~~Objektteil~~~~\r\n\r\n\r\n\r\n~~Objektnummer~~-1~~\r\n\r\n\r\n\r\n~~Einsatzart~~Hilfeleistungseinsatz~~\r\n\r\n\r\n\r\n~~Alarmgrund~~H:Natur~~\r\n\r\n\r\n\r\n~~Sondersignal~~ohne Sondersignal~~\r\n\r\n\r\n\r\n~~Einsatznummer~~322088295~~\r\n\r\n\r\n\r\n~~Besonderheiten~~TESTETESTTESTE~~\r\n\r\n\r\n\r\n~~Name~~,~~\r\n\r\n\r\n\r\n~~EMListe~~FL BRB 01/16-21, RLS BRB DGL 2~~\r\n\r\n\r\n\r\n~~Status~~Fahrzeug~~Zuget~~Alarm~~Ausger=C3=BCckt~~\r\n\r\n\r\n\r\n~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8~~FL BRB 01/16-21~~08:21~~=\r\n~~\r\n\r\n\r\n\r\n~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8~~FL BRB 01/16-21~~08:21~~=\r\n~~\r\n\r\n\r\n\r\n~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=C3=B8~~RLS BRB DGL 2~~08:23~~~~\r\n\r\n\r\n\r\n~~WGS84_X~~52.33823333~~\r\n\r\n\r\n\r\n~~WGS84_Y~~12.48626667~~\r\n\r\n\r\n\r\n~~Koord_EPSG_25833~~12.48626667~~52.33823333~~\r\n\r\n\r\n\r\n~~Koord_EPSG_4326~~E1248630~~N5233820~~~~Einsatzortzusatz~~~~\r\n\r\n\r\n\r\n~~Alarmzeit~~29.09.22&08:23~~\r\n";

#[test]
fn test_get_message_body_plain_text() {
    let example = Message {
        uid: Some(1),
        header: Some(HEADER_CONTENT_TYPE.as_bytes().to_vec()),
        text: Some(BODY_PLAIN_TEXT.as_bytes().to_vec()),
    };
    let body = get_message_body(example);
    assert_eq!(body, Some(BODY_PLAIN_TEXT.to_string()));
}

#[test]
pub fn test_get_message_body_multipart() {
    // the get_message_body function should return the plain text part of the multipart mail
    let example = Message {
        uid: Some(1),
        header: Some(HEADER_CONTENT_TYPE_MULTIPART.as_bytes().to_vec()),
        text: Some(MULTIPART_BODY.as_bytes().to_vec()),
    };
    let body = get_message_body(example);
    assert_eq!(body, Some(MULTIPART_BODY_PLAIN_TEXT.to_string()));
}
