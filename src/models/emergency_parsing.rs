use std::{
    cmp::max,
    iter::Peekable,
    str::{Chars, FromStr},
};

use chrono::NaiveDateTime;
use log::{info, trace, warn};

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

#[derive(Debug, Clone, Copy)]
struct AlarmTableIndices {
    unit: usize,
    station: usize,
    alarm_time: usize,
    header_count: usize,
}

//
// ------------------ Parsing ------------------
//

impl FromStr for Emergency {
    type Err = String;

    // #[no_panic]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ems = Emergency::default();
        let mut line_nr = 1; // line numbers start at 1 (not 0!!!, lol)

        let mut in_stream = s.chars().peekable();
        let mut header_indicies: Option<AlarmTableIndices> = None;

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

                "Status" => {
                    if let Some(_) = header_indicies {
                        warn!("Found multiple alarm table headers in line {}!", line_nr);
                    }

                    header_indicies = Some(parse_alarm_table_header(&mut in_stream, &mut line_nr));
                    continue; // skip the line end ~~ skip, since we already parsed it
                }
                "ALARM" => {
                    if let None = header_indicies {
                        warn!(
                            "Found alarm table entry before header in line {}! skipping!",
                            line_nr
                        );
                        continue;
                    } else if let Some(headers) = header_indicies {
                        parse_alarm_table_entry(&mut in_stream, headers, &mut ems, &mut line_nr);
                        continue; // skip the line end ~~ skip, since we already parsed it
                    }
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
                    let addition = read_value(&mut in_stream);
                    ems.location_addition = (!addition.is_empty()).then(|| addition);
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

fn parse_alarm_table_header(
    in_stream: &mut Peekable<Chars<'_>>,
    line_nr: &mut u64,
) -> AlarmTableIndices {
    let mut indices = AlarmTableIndices {
        unit: 0,
        station: 0,
        alarm_time: 0,
        header_count: 0,
    };

    // at the start we are in the ~~Status~~ line right after the Status~~ part.
    // parse header for column indices
    while in_stream.peek().is_some() {
        let peek = in_stream.peek();
        if let Some(peek) = peek {
            if "\r\n".contains(*peek) {
                break; // end of line, => end of header
            }
        }

        let col = read_value(in_stream);

        match col.as_str() {
            "Fahrzeug" => indices.unit = indices.header_count,
            "Zuget" => indices.unit = indices.header_count,
            "Wache" => indices.station = indices.header_count,
            "Alarm" => indices.alarm_time = indices.header_count,
            "Alarmiert" => indices.alarm_time = indices.header_count,
            "Tableau-Adresse" => {}
            "Ausgerückt" => {}
            _ => {
                trace!("Unused column {} in alarm table in line {}!", col, *line_nr);
            }
        }
        indices.header_count += 1;

        let end_section = expect_literal(in_stream, "~~", *line_nr);
        if let Err(e) = end_section {
            warn!("missing '~~' to end alarm table header: {}", e);
            continue;
        }
    }

    println!("header: {:?}", indices);
    return indices;
}

fn vec_remove_replace(vec: &mut Vec<String>, index: usize) -> String {
    let elem = vec.remove(index);
    vec.insert(index, String::new());
    return elem;
}

fn parse_alarm_table_entry(
    in_stream: &mut Peekable<Chars<'_>>,
    headers: AlarmTableIndices,
    ems: &mut Emergency,
    line_nr: &mut u64,
) {
    // in_stream is directly after the ALARM~~ part
    let mut entries: Vec<String> = Vec::with_capacity(headers.header_count);

    while in_stream.peek().is_some() {
        let peek = in_stream.peek();
        if let Some(peek) = peek {
            if "\r\n".contains(*peek) {
                break; // end of line, => end of entry
            }
        }

        let entry = read_value(in_stream);
        entries.push(entry);

        let end_section = expect_literal(in_stream, "~~", *line_nr);
        if let Err(e) = end_section {
            warn!("missing '~~' to end alarm table entry: {}", e);
            skip_line(in_stream, line_nr);
            return;
        }
    }
    while entries.len() < headers.header_count {
        entries.push(String::new());
    }

    // parse entry:
    if entries.len() <= max(headers.unit, max(headers.station, headers.alarm_time)) {
        warn!(
            "Insufficent amount of alarm table entry values  ({}) columns in line {}!",
            entries.len(),
            *line_nr
        );
        skip_line(in_stream, line_nr);
        return;
    }
    entries[headers.station] = entries[headers.station]
        .trim_end_matches(|c: char| !c.is_ascii())
        .to_string();

    let id_str = vec_remove_replace(&mut entries, headers.unit);
    let id = RadioIdentifier::from_str(&id_str);
    let id = match id {
        Ok(id) => Either::Left(id),
        Err(e) => {
            trace!(
                "Failed to parse RadioIdentifier {} in line {}: {}, using bare.",
                entries[headers.unit],
                *line_nr,
                e
            );
            Either::Right(id_str)
        }
    };
    if entries[headers.station].is_empty() && entries[headers.alarm_time].is_empty() {
        // this is an empty table entry, skip the entry, to display no units it
        trace!("empty alarm table entry in line {}!", *line_nr);
        return;
    }
    let unit = UnitAlarmTime {
        unit_id: id,
        station: vec_remove_replace(&mut entries, headers.station),
        alarm_time: vec_remove_replace(&mut entries, headers.alarm_time),
    };
    ems.unit_alarm_times.push(unit);
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
