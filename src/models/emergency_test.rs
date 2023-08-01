use std::str::FromStr;

use chrono::NaiveDate;

use crate::{
    config::logging,
    models::{either::Either, emergency::Emergency, radio_identifier::RadioIdentifier},
};

#[cfg(test)]
const TEST_MAIL_CONTENT: &str = r#"
~~Ort~~Brandenburg an der Havel~~

~~Ortsteil~~Göttin/BRB~~

~~Ortslage~~Görisgräben~~

~~Strasse~~Görisgräben~~

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

~~Status~~Fahrzeug~~Zuget~~Alarm~~Ausgerückt~~

~~ALARM~~unbekannt#~~BRB FW Brandenburg 1ø~~FL BRB 01/16-21~~08:21~~~~

~~ALARM~~unbekannt#~~BRB FW Brandenburg 1ø~~FL BRB 01/16-21~~08:21~~~~

~~ALARM~~unbekannt#~~BRB FW Brandenburg 1ø~~RLS BRB DGL 2~~08:23~~~~

~~WGS84_X~~52.33823333~~

~~WGS84_Y~~12.48626667~~

~~Koord_EPSG_25833~~12.48626667~~52.33823333~~

~~Koord_EPSG_4326~~E1248630~~N5233820~~~~Einsatzortzusatz~~~~

~~Alarmzeit~~29.09.22&08:23~~
"#;

#[test]
fn test_parse_emergency() {
    logging::init_logging();
    let ems = Emergency::from_str(TEST_MAIL_CONTENT).unwrap();

    assert_eq!(ems.town, "Brandenburg an der Havel".to_string());
    assert_eq!(ems.district, "Göttin/BRB".to_string());
    assert_eq!(ems.location, "Görisgräben".to_string());
    assert_eq!(ems.street, "Görisgräben".to_string());
    assert_eq!(ems.house_number, "22".to_string());
    assert_eq!(ems.object, None);
    assert_eq!(ems.fire_department_plan, None);
    assert_eq!(ems.object_part, None);
    assert_eq!(ems.object_number, None);
    assert_eq!(ems.emergency_type, "Hilfeleistungseinsatz".to_string());
    assert_eq!(ems.keyword, "H:Natur".to_string());
    assert_eq!(ems.code3, "ohne Sondersignal".to_string());
    assert_eq!(ems.emergency_number, 322088295);
    assert_eq!(ems.note, Some("TESTETESTTESTE".to_string()));
    assert_eq!(ems.patient_name, None);
    assert!(ems
        .dispatched_units
        .iter()
        .zip(vec![
            Either::Left("FL BRB 01/16-21".parse::<RadioIdentifier>().unwrap()),
            Either::Right("RLS BRB DGL 2".to_string())
        ])
        .all(|(a, b)| *a == b));
    assert_eq!(ems.unit_alarm_times.len(), 3);
    assert_eq!(ems.unit_alarm_times[0].alarm_time, "08:21");
    assert_eq!(
        ems.unit_alarm_times[0].unit_id,
        Either::Left("FL BRB 01/16-21".parse::<RadioIdentifier>().unwrap())
    );
    assert_eq!(ems.unit_alarm_times[0].station, "BRB FW Brandenburg 1ø");
    assert_eq!(ems.unit_alarm_times[1].alarm_time, "08:21");
    assert_eq!(
        ems.unit_alarm_times[1].unit_id,
        Either::Left("FL BRB 01/16-21".parse::<RadioIdentifier>().unwrap())
    );
    assert_eq!(ems.unit_alarm_times[1].station, "BRB FW Brandenburg 1ø");
    assert_eq!(ems.unit_alarm_times[2].alarm_time, "08:23");
    assert_eq!(
        ems.unit_alarm_times[2].unit_id,
        Either::Right("RLS BRB DGL 2".to_string())
    );
    assert_eq!(ems.unit_alarm_times[2].station, "BRB FW Brandenburg 1ø");
    assert_eq!(
        ems.alarm_time,
        NaiveDate::from_ymd(2022, 9, 29).and_hms(8, 23, 0)
    );
}
