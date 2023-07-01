use std::{
    iter::Peekable,
    str::{Chars, FromStr},
};

use chrono::NaiveDateTime;
use log::{info, warn};

use crate::models::{
    either::Either, radio_identifier::RadioIdentifier, unit_alarm_time::UnitAlarmTime,
};

use super::emergency::Emergency;

//
// ------------------ Helper functions ------------------
//

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

fn skip_line(chars: &mut Peekable<Chars>, line_nr: &mut u64) -> () {
    while let Some(next) = chars.peek() {
        if next == &'\n' {
            *line_nr += 1;
            break;
        }
        let _ = chars.next();
    }
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

macro_rules! simple_property {
    ($s:expr, $($p:ident).+) => {
        $($p).+ = read_value(&mut $s)
    };
}

macro_rules! check_error_skip_line {
    ($f: expr, $s: ident, $l: ident) => {
        match $f {
            Err(e) => {
                warn!("{}", e);
                skip_line(&mut $s, &mut $l);
                continue;
            }
            _ => {}
        }
    };
}

//
// ------------------ Parsing ------------------
//

impl FromStr for Emergency {
    type Err = String;

    // #[no_panic]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ems = Emergency::default();
        let mut line_nr = 0;

        let mut in_stream = s.chars().peekable();

        while in_stream.peek().is_some() {
            skip_whitespace_count_lines(&mut in_stream, &mut line_nr);
            if in_stream.peek().is_none() {
                break;
            }
            check_error_skip_line!(
                expect_literal(&mut in_stream, "~~", line_nr),
                in_stream,
                line_nr
            );

            let property = read_value(&mut in_stream);

            check_error_skip_line!(
                expect_literal(&mut in_stream, "~~", line_nr),
                in_stream,
                line_nr
            );

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
                    parse_dispatched_units(&mut in_stream, &mut ems);
                }

                "Status" => check_error_skip_line!(
                    expect_literal(
                        &mut in_stream,
                        "Fahrzeug~~Zuget~~Alarm~~Ausgerückt", // TODO: make dynamic, implement encoding correctly
                        line_nr,
                    ),
                    in_stream,
                    line_nr
                ),
                "ALARM" => {
                    let _status = read_value(&mut in_stream);
                    check_error_skip_line!(
                        expect_literal(&mut in_stream, "~~", line_nr),
                        in_stream,
                        line_nr
                    );
                    let unit = read_value(&mut in_stream);
                    check_error_skip_line!(
                        expect_literal(&mut in_stream, "~~", line_nr),
                        in_stream,
                        line_nr
                    );
                    let id = read_value(&mut in_stream);
                    check_error_skip_line!(
                        expect_literal(&mut in_stream, "~~", line_nr),
                        in_stream,
                        line_nr
                    );
                    let alarm_time = read_value(&mut in_stream);
                    check_error_skip_line!(
                        expect_literal(&mut in_stream, "~~", line_nr),
                        in_stream,
                        line_nr
                    );
                    let _responding = read_value(&mut in_stream);
                    let unit_alarm_time = UnitAlarmTime::from_values(id, unit, alarm_time);
                    ems.unit_alarm_times.push(unit_alarm_time);
                }

                "WGS84_X" => {
                    let _ = read_value(&mut in_stream);
                }
                "WGS84_Y" => {
                    let _ = read_value(&mut in_stream);
                }
                "Koord_EPSG_25833" => {
                    let _ = read_value(&mut in_stream);
                    check_error_skip_line!(
                        expect_literal(&mut in_stream, "~~", line_nr),
                        in_stream,
                        line_nr
                    );
                    let _ = read_value(&mut in_stream);
                }
                "Koord_EPSG_4326" => {
                    let _ = read_value(&mut in_stream);
                    check_error_skip_line!(
                        expect_literal(&mut in_stream, "~~", line_nr),
                        in_stream,
                        line_nr
                    );
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
                    skip_line(&mut in_stream, &mut line_nr);
                    // skip value. If this is not a simple key value pair, we might be screwed...
                    // TODO: perhaps expect a new line also (since all but one property are followed by a new line)
                    warn!(
                        "Unknown property {} detected in line {}!",
                        property, line_nr
                    );
                }
            }

            check_error_skip_line!(
                expect_literal(&mut in_stream, "~~", line_nr),
                in_stream,
                line_nr
            ); // found at the end of each line
        }

        return Ok(ems);
    }
}

fn parse_dispatched_units(in_stream: &mut Peekable<Chars<'_>>, ems: &mut Emergency) {
    let em_string = read_value(in_stream);
    em_string.split(", ").for_each(|em| {
        let identifier = RadioIdentifier::from_str(em);
        if let Ok(identifier) = identifier {
            ems.dispatched_units.push(Either::Left(identifier));
        } else {
            info!(
                "Failed to parse RadioIdentifier {} using as bare Identifier!",
                em
            );
            ems.dispatched_units.push(Either::Right(em.to_string()));
        }
    });
}
