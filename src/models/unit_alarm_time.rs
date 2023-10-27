use std::str::FromStr;

use super::{either::Either, radio_identifier::RadioIdentifier};

#[derive(Debug, Eq, PartialEq)]
pub struct UnitAlarmTime {
    pub unit_id: Either<RadioIdentifier, String>,
    pub station: String,
    pub alarm_time: String,
}

impl UnitAlarmTime {
    pub fn from_values(unit_id: String, station: String, alarm_time: String) -> Self {
        let radio_id = RadioIdentifier::from_str(&unit_id);

        return match radio_id {
            Ok(radio_id) => UnitAlarmTime {
                unit_id: radio_id.to_left(),
                station,
                alarm_time,
            },
            Err(_) => UnitAlarmTime {
                unit_id: Either::Right(unit_id),
                station,
                alarm_time,
            },
        };
    }
}
