use chrono::NaiveDateTime;
use log::{info, warn};
use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use super::{either::Either, radio_identifier::RadioIdentifier};

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
    pub unit_alarm_times: Vec<String>,
    pub alarm_time: NaiveDateTime,
}

fn skip_whitespace_count_lines(chars: &mut Peekable<Chars>, lines: &mut u64) -> () {
    while let Some(next) = chars.peek() {
        // check next character without advancing, so that a non-whitespace
        // is not consumed and therefore availible for the next step (e.g. parsing values)
        if !next.is_whitespace() {
            break;
        }

        if next == &'\n' {
            *lines += 1;
        }

        let _ = chars.next(); // advance iterator
    }
}

fn read_value(chars: &mut Peekable<Chars>) -> String {
    let mut value = String::new();
    while let Some(next) = chars.peek() {
        if next == &'~' {
            break;
        }
        value.push(*next);
        let _ = chars.next();
    }
    return value;
}

fn expect_literal(chars: &mut Peekable<Chars>, literal: &str, line_nr: u64) -> Result<(), String> {
    for c in literal.chars() {
        let Some(current_char) = chars.next() else {
            return Err(format!("Expected literal \"{}\" in line {}, got eoi instead.", literal, line_nr));
        };
        if c != current_char {
            return Err(format!(
                "Expected \'{}\' in literal {} in line {}, got \'{}\' instead.",
                c, literal, line_nr, current_char
            ));
        }
    }

    return Ok(());
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

macro_rules! simple_property {
    ($s:expr, $($p:ident).+) => {
        $($p).+ = read_value(&mut $s)
    };
}

impl FromStr for Emergency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ems = Emergency::default();
        let mut line_nr = 0;

        let mut in_stream = s.chars().peekable();
        while in_stream.peek().is_some() {
            skip_whitespace_count_lines(&mut in_stream, &mut line_nr);
            if in_stream.peek().is_none() {
                println!("eoi");
                break;
            }
            expect_literal(&mut in_stream, "~~", line_nr)?;

            let property = read_value(&mut in_stream);

            println!("property: {}", property.as_str());
            expect_literal(&mut in_stream, "~~", line_nr)?;

            match property.as_str() {
                "Ort" => {
                    simple_property!(in_stream, ems.town);
                }
                "Ortsteil" => {
                    simple_property!(in_stream, ems.district);
                }
                "Ortslage" => {
                    simple_property!(in_stream, ems.location);
                }
                "Strasse" => {
                    simple_property!(in_stream, ems.street);
                }
                "Hausnummer" => {
                    simple_property!(in_stream, ems.house_number);
                }
                "Objekt" => {
                    let obj = read_value(&mut in_stream);
                    ems.object = (!obj.is_empty()).then(|| obj);
                }
                "FWPlan" => {
                    let plan_nr = read_value(&mut in_stream);
                    ems.fire_department_plan = (!plan_nr.is_empty()).then(|| plan_nr);
                }
                "Objektteil" => {
                    let obj_part = read_value(&mut in_stream);
                    ems.object_part = (!obj_part.is_empty()).then(|| obj_part);
                }
                "Objektnummer" => {
                    if let Ok(number) = read_value(&mut in_stream).parse::<i64>() {
                        if number != -1 {
                            ems.object_number = Some(number);
                        }
                    } else {
                        warn!("failed to convert Objektnummer to i64 in line {}", line_nr);
                    }
                }
                "Einsatzart" => {
                    simple_property!(in_stream, ems.emergency_type);
                }
                "Alarmgrund" => {
                    simple_property!(in_stream, ems.keyword);
                }
                "Sondersignal" => {
                    simple_property!(in_stream, ems.code3);
                }
                "Einsatznummer" => {
                    // parse the number as u64, if it fails, set it to 0
                    // and warn (recovering is always better than crashing here, since we want to print a fax
                    // even if not everything could be parsed)
                    if let Ok(number) = read_value(&mut in_stream).parse::<u64>() {
                        ems.emergency_number = number;
                    } else {
                        warn!(
                            "failed to convert Einsatznummer integer in line {}",
                            line_nr
                        );
                        ems.emergency_number = 0;
                    }
                }

                "Besonderheiten" => {
                    let note = read_value(&mut in_stream);
                    ems.note = (!note.is_empty()).then(|| note);
                }
                "Name" => {
                    let patient = read_value(&mut in_stream);
                    if !patient.is_empty() && patient != "," {
                        ems.patient_name = Some(patient);
                    }
                }
                "EMListe" => {
                    let em_string = read_value(&mut in_stream);
                    println!("em_string: {}", em_string);
                    em_string.split(", ").for_each(|em| {
                        let identifier = RadioIdentifier::from_str(em);
                        if let Ok(identifier) = identifier {
                            ems.dispatched_units.push(Either::Left(identifier));
                        } else {
                            info!(
                                "Failed to parse RadioIdentifier {} in line {}, using as bare Identifier!",
                                em, line_nr
                            );
                            ems.dispatched_units.push(Either::Right(em.to_string()));
                        }
                    });
                }

                "Status" => expect_literal(
                    &mut in_stream,
                    "Fahrzeug~~Zuget~~Alarm~~Ausgerückt", // TODO: make dynamic, implement encoding correctly
                    line_nr,
                )?,
                "ALARM" => {
                    let _status = read_value(&mut in_stream);
                    expect_literal(&mut in_stream, "~~", line_nr)?;
                    let _unit = read_value(&mut in_stream);
                    expect_literal(&mut in_stream, "~~", line_nr)?;
                    let _id = read_value(&mut in_stream);
                    expect_literal(&mut in_stream, "~~", line_nr)?;
                    let alarm_time = read_value(&mut in_stream);
                    ems.unit_alarm_times.push(alarm_time);
                    expect_literal(&mut in_stream, "~~", line_nr)?;
                    let _responding = read_value(&mut in_stream);
                    // TODO: check if the units field should be created from this list, with more info (such as alarm time and department)
                    // would require duplicate checking, since some units may appear multiple times
                }

                "WGS84_X" => {
                    let _ = read_value(&mut in_stream);
                }
                "WGS84_Y" => {
                    let _ = read_value(&mut in_stream);
                }
                "Koord_EPSG_25833" => {
                    let _ = read_value(&mut in_stream);
                    expect_literal(&mut in_stream, "~~", line_nr)?;
                    let _ = read_value(&mut in_stream);
                }
                "Koord_EPSG_4326" => {
                    let _ = read_value(&mut in_stream);
                    expect_literal(&mut in_stream, "~~", line_nr)?;
                    let _ = read_value(&mut in_stream);
                }
                "Einsatzortzusatz" => {
                    let _ = read_value(&mut in_stream);
                }
                "Alarmzeit" => {
                    let time_str = read_value(&mut in_stream);
                    if let Ok(time) = NaiveDateTime::parse_from_str(&time_str, "%d.%m.%y&%H:%M") {
                        ems.alarm_time = time;
                    } else {
                        warn!(
                            "Failed to parse alarm time {} in line {}!",
                            time_str, line_nr
                        );
                    }
                }

                _ => {
                    let _ = read_value(&mut in_stream);
                    // skip value. If this is not a simple key value pair, we might be screwed...
                    // TODO: perhaps expect a new line also (since all but one property are followed by a new line)
                    warn!(
                        "Unknown property {} detected in line {}!",
                        property, line_nr
                    );
                }
            }

            expect_literal(&mut in_stream, "~~", line_nr)?; // found at the end of each line

            // println!("{}", value.as_str());
            println!("{:?}", in_stream.peek());
        }

        return Ok(ems);
    }
}
