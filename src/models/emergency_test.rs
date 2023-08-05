use std::str::FromStr;

use chrono::NaiveDate;

use crate::{
    config::logging,
    models::{either::Either, emergency::Emergency, radio_identifier::RadioIdentifier},
};

#[cfg(test)]
const TEST_MAIL_CONTENT: &str = include_str!("../../examples/emergency_simple.txt");

#[test]
#[allow(deprecated)]
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
    assert_eq!(ems.unit_alarm_times[0].station, "BRB FW Brandenburg 1");
    assert_eq!(ems.unit_alarm_times[1].alarm_time, "08:22");
    assert_eq!(
        ems.unit_alarm_times[1].unit_id,
        Either::Left("FL BRB 01/16-21".parse::<RadioIdentifier>().unwrap())
    );
    assert_eq!(ems.unit_alarm_times[1].station, "BRB FW Brandenburg 1");
    assert_eq!(ems.unit_alarm_times[2].alarm_time, "08:23");
    assert_eq!(
        ems.unit_alarm_times[2].unit_id,
        Either::Right("RLS BRB DGL 2".to_string())
    );
    assert_eq!(ems.unit_alarm_times[2].station, "BRB FW Brandenburg 1");
    assert_eq!(
        ems.alarm_time,
        NaiveDate::from_ymd(2022, 9, 29).and_hms(8, 23, 0)
    );
}
