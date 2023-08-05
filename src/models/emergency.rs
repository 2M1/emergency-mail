use chrono::NaiveDateTime;

use super::{either::Either, radio_identifier::RadioIdentifier, unit_alarm_time::UnitAlarmTime};

#[derive(Debug, Default)]
pub struct Emergency {
    pub town: String,
    pub district: String,
    pub location: String,
    pub location_addition: Option<String>,
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

    pub fn address_text(&self) -> String {
        let mut s = String::new();
        if self.object.is_some() {
            s.push_str(&self.object.as_ref().unwrap());
            s.push_str("\n");
        }
        s.push_str(&self.street);
        s.push_str(" ");
        s.push_str(&self.house_number);
        s.push_str("\n");
        s.push_str(&self.district);

        if !self.location.is_empty() && self.location != self.district {
            s.push_str("\n");
            s.push_str(&self.location);
        }

        // if let Some(o) = &self.object_part {
        //     s.push_str(" ");
        //     s.push_str(o);
        // }
        // if let Some(o) = &self.object_number {
        //     s.push_str(" ");
        //     s.push_str(&o.to_string());
        // }

        return s;
    }

    pub fn get_patient_name(&self) -> Option<String> {
        if let Some(name) = &self.patient_name {
            let (last, first) = name.split_at(name.find(",").unwrap_or(name.len()));
            let first = &first[1..]; // the , of the first name is now the first char of first
            return Some(format!("{} {}", first, last));
        } else {
            return None;
        }
    }
}
