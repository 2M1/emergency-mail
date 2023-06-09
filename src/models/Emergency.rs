use std::{marker::PhantomData, str::FromStr};

use crate::models::Builder::Builder;

#[derive(Debug)]
pub struct RadioIdentifier {
    pub org: String,
    pub county: String,
    pub agency: u64,
    pub car_type: u64,
    pub number: u64,
}

#[derive(Debug)]
pub struct Emergency {
    pub town: String,
    pub district: String,
    pub location: String,
    pub street: String,
    pub house_number: String,
    pub object: String,
    pub fire_department_plan: String,
    pub object_part: String,
    pub object_number: i64,
    pub emergency_type: String,
    pub keyword: String,
    pub emergency_number: u64,
    pub note: String,
    pub patient_name: String,
    pub dispatched_units: Vec<RadioIdentifier>,
    pub unit_alarm_times: Vec<String>,
    pub alarm_time: String,
}

#[derive(Debug, Default)]
struct EmergencyBuilder {
    pub town: Option<String>,
    pub district: Option<String>,
    pub location: Option<String>,
    pub street: Option<String>,
    pub house_number: Option<String>,
    pub object: Option<String>,
    pub fire_department_plan: Option<String>,
    pub object_part: Option<String>,
    pub object_number: Option<i64>,
    pub emergency_type: Option<String>,
    pub keyword: Option<String>,
    pub emergency_number: Option<u64>,
    pub note: Option<String>,
    pub patient_name: Option<String>,
    pub dispatched_units: Option<Vec<RadioIdentifier>>,
    pub unit_alarm_times: Option<Vec<String>>,
    pub alarm_time: Option<String>,
}

impl EmergencyBuilder {
    fn new() -> EmergencyBuilder {
        return EmergencyBuilder::default();
    }
}

impl Builder<Emergency> for EmergencyBuilder {
    type E = String;

    fn build(&self) -> Result<Emergency, Self::E> {
        return Err(String::from("Not implemented"));
    }
}

fn consume<'a>(s: &'a str, prefix: &str) -> Result<&'a str, ()> {
    if s.starts_with(prefix) {
        return Ok(&s[prefix.len()..]);
    } else {
        return Err(());
    }
}

impl FromStr for Emergency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut builder = EmergencyBuilder::new();
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
                        builder.town = Some(value.to_string());
                    }
                    "Ortsteil" => {
                        builder.district = Some(value.to_string());
                    }
                    "Ortslage" => {
                        builder.location = Some(value.to_string());
                    }
                    "Strasse" => {
                        builder.street = Some(value.to_string());
                    }
                    "Hausnummer" => {
                        builder.house_number = Some(value.to_string());
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
                        builder.emergency_type = Some(value.to_string());
                    }
                    "Alarmgrund" => {
                        builder.keyword = Some(value.to_string());
                    }
                    "Einsatznummer" => {
                        builder.emergency_number = Some(value.parse::<u64>().map_err(|_e| {
                            format!(
                                "Error in line {}: Could not parse {} as u64",
                                line_nr, value
                            )
                        })?);
                    }
                    "Besonderheiten" => {
                        builder.note = Some(value.to_string());
                    }
                    "Name" => {
                        builder.patient_name = Some(value.to_string());
                    }
                    "EMListe" => {}
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
