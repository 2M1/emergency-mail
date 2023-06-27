use chrono::NaiveDateTime;
use log::{info, warn};
use no_panic::no_panic;
use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use super::{either::Either, radio_identifier::RadioIdentifier, unit_alarm_time::UnitAlarmTime};

#[derive(Debug, Default)]
pub struct Emergency {
    pub town: String,
    pub district: String,
    pub location: String,
    pub street: String,
    pub house_number: String,
    pub object: Option<String>,
    pub fire_department_plan: Option<String>,
    pub object_part: Option<String>,
    pub object_number: Option<i64>,
    pub emergency_type: String,
    pub keyword: String,
    pub code3: String,
    pub emergency_number: u64,
    pub note: Option<String>,
    pub patient_name: Option<String>,
    pub dispatched_units: Vec<Either<RadioIdentifier, String>>,
    pub unit_alarm_times: Vec<UnitAlarmTime>,
    pub alarm_time: NaiveDateTime,
}

impl Emergency {
    fn verify_minimum_fields(&mut self) -> bool {
        return !self.town.is_empty()
            && !self.location.is_empty()
            && !self.street.is_empty()
            && !self.house_number.is_empty()
            && !self.emergency_type.is_empty()
            && !self.keyword.is_empty()
            && !self.code3.is_empty()
            && !self.dispatched_units.is_empty()
            && !self.unit_alarm_times.is_empty()
            && self.emergency_number != 0;
    }

    fn count_units_from_town(&self, town: u8) -> u64 {
        let mut n = 0;
        for u in self.dispatched_units.iter() {
            if let Either::Left(id) = u {
                if id.agency == town {
                    n += 1;
                }
            }
        }
        return n;
    }
}
