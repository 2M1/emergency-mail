use std::str::FromStr;

use crate::models::emergency::Emergency;

#[cfg(test)]
const TEST_MAIL_CONTENT: &str = r#"
~~Ort~~Brandenburg an der Havel~~

~~Ortsteil~~G=F6ttin/BRB~~

~~Ortslage~~G=F6risgr=E4ben~~

~~Strasse~~G=F6risgr=E4ben~~

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

~~Status~~Fahrzeug~~Zuget~~Alarm~~Ausger=FCckt~~

~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=F8~~FL BRB 01/16-21~~08:21~~~~

~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=F8~~FL BRB 01/16-21~~08:21~~~~

~~ALARM~~unbekannt#~~BRB FW Brandenburg 1=F8~~RLS BRB DGL 2~~08:23~~~~

~~WGS84_X~~52.33823333~~

~~WGS84_Y~~12.48626667~~

~~Koord_EPSG_25833~~12.48626667~~52.33823333~~

~~Koord_EPSG_4326~~E1248630~~N5233820~~~~Einsatzortzusatz~~~~

~~Alarmzeit~~29.09.22&08:23~~
"#;

#[test]
fn test_parse_emergency() {
    let ems = Emergency::from_str(TEST_MAIL_CONTENT).unwrap();
    assert_eq!(ems.town, "Brandenburg an der Havel".to_string());
}
