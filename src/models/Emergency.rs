use chrono::NaiveDate;
use log::warn;
use std::str::FromStr;

use super::RadioIdentifier::RadioIdentifier;
use crate::models::Builder::Builder;

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
    pub dispatched_units: Vec<RadioIdentifier>,
    pub unit_alarm_times: Vec<String>,
    pub alarm_time: NaiveDate,
}

fn consume<'a>(s: &'a str, prefix: &str) -> Result<&'a str, ()> {
    if s.starts_with(prefix) {
        return Ok(&s[prefix.len()..]);
    } else {
        return Err(());
    }
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
}

impl FromStr for Emergency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut builder = Emergency::default();
        let lines = s.lines();
        let mut line_nr = 0;

        for current_line in lines {
            line_nr += 1;
            let current_line = current_line.trim();
            if current_line == "" {
                continue;
            }

            let mut parts = current_line.split("~~").peekable();

            while parts.peek().is_some() {
                let Some(property) = parts.next() else {
                    return Err(format!("Error in line {}: Empty property", line_nr));  
                    // should not happen, because of the while peek loop
                };

                let Some(value) = parts.next() else {
                    return Err(format!("Error in line {}: Missing value for property {}", line_nr, property));
                };

                match property {
                    "Ort" => {
                        builder.town = value.to_string();
                    }
                    "Ortsteil" => builder.district = value.to_string(),
                    "Ortslage" => {
                        builder.location = value.to_string();
                    }
                    "Strasse" => {
                        builder.street = value.to_string();
                    }
                    "Hausnummer" => {
                        builder.house_number = value.to_string();
                    }
                    "Objekt" => {
                        builder.object = Some(value.to_string());
                    }
                    "FWPlan" => {
                        builder.fire_department_plan = Some(value.to_string());
                    }
                    "Objektteil" => {
                        builder.object_part = Some(value.to_string());
                    }
                    "Objektnummer" => {
                        builder.object_number = Some(value.parse::<i64>().map_err(|_e| {
                            format!(
                                "Error in line {}: Could not parse {} as i64",
                                line_nr, value
                            )
                        })?);
                    }
                    "Einsatzart" => {
                        builder.emergency_type = value.to_string();
                    }
                    "Alarmgrund" => {
                        builder.keyword = value.to_string();
                    }
                    "Sondersignal" => {
                        builder.code3 = value.to_string();
                    }
                    "Einsatznummer" => {
                        builder.emergency_number = value.parse::<u64>().map_err(|_e| {
                            format!(
                                "Error in line {}: Could not parse {} as u64",
                                line_nr, value
                            )
                        })?;
                    }
                    "Besonderheiten" => {
                        builder.note = Some(value.to_string());
                    }
                    "Name" => {
                        builder.patient_name = Some(value.to_string());
                    }
                    "EMListe" => {}
                    _ => {
                        warn!("Unknown property {} in line {}", property, line_nr);
                    }
                }

                let Some(spacer) = parts.next() else {
                    continue;
                };
                if !spacer.is_empty() {
                    return Err(format!(
                        "Error in line {}: Expected four ~ between two properties., got {}",
                        line_nr, spacer
                    ));
                }
            }
        }

        return Err(String::from("Not implemented"));
    }
}
