use chrono::NaiveDate;
use log::warn;
use std::{
    iter::Peekable,
    ops::{Add, Deref},
    str::{Chars, FromStr},
    time::Instant,
};

use crate::unrecoverable;

use super::{radio_identifier::RadioIdentifier, recoverable::Recoverable};

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
        print!("testing {:?}", c);
        let Some(current_char) = chars.next() else {
            return Err(format!("Expected literal \"{}\" in line {}, got eoi instead.", literal, line_nr));
        };
        println!(" against {:?}", current_char);
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
        let mut builder = Emergency::default();
        let mut line_nr = 0;

        let mut in_stream = s.chars().peekable();
        while in_stream.peek().is_some() {
            skip_whitespace_count_lines(&mut in_stream, &mut line_nr);
            expect_literal(&mut in_stream, "~~", line_nr)?;

            println!("{:?}", in_stream.peek());
            let property = read_value(&mut in_stream);

            println!("{}", property.as_str());
            expect_literal(&mut in_stream, "~~", line_nr)?;

            match property.as_str() {
                "Ort" => {
                    simple_property!(in_stream, builder.town);
                    println!("{}", builder.town);
                }
                "Ortsteil" => {
                    simple_property!(in_stream, builder.district);
                    println!("{}", builder.district);
                }
                "Ortslage" => {
                    simple_property!(in_stream, builder.location);
                    println!("{}", builder.location);
                }

                "Status" => expect_literal(
                    &mut in_stream,
                    "Fahrzeug~~Zuget~~Alarm~~Ausger=FCckt",
                    line_nr,
                )?,

                _ => {
                    let _ = read_value(&mut in_stream);
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

        return Err(String::from("Not implemented"));
    }
}
