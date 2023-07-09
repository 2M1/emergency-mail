use std::{cmp::max, path::Iter};

use crate::models::{either::Either, emergency::Emergency};

use super::{
    print_xps::print_test,
    xps_document::XPSSingleDocument,
    xps_page::{DrawingAttributes, Point, XPSPage, LINE_HEIGHT},
};

const LABEL_OFFSET: f32 = 180.0;
const SECTION_OFFSET: f32 = 150.0;
const CHAR_WIDTH_40: f32 = 25.0;

pub fn print_emergency(ems: Emergency) {
    let doc = create_emergency_xps(&ems);
    // print_test(doc);
    doc.safe();
}

fn add_emergency_header_section(ems: &Emergency, page: &mut XPSPage) {
    // create header blocks

    page.add_outline_polygon(
        &[
            Point {
                x: SECTION_OFFSET,
                y: 250.0,
            },
            Point {
                x: SECTION_OFFSET,
                y: 400.0,
            },
            Point { x: 500.0, y: 400.0 },
            Point { x: 500.0, y: 250.0 },
        ],
        DrawingAttributes::DEFAULT,
    );
    page.add_outline_polygon(
        &[
            Point { x: 500.0, y: 250.0 },
            Point { x: 500.0, y: 400.0 },
            Point { x: 780.0, y: 400.0 },
            Point { x: 780.0, y: 250.0 },
        ],
        DrawingAttributes::DEFAULT,
    );
    page.add_outline_polygon(
        &[
            Point { x: 780.0, y: 250.0 },
            Point { x: 780.0, y: 400.0 },
            Point {
                x: 1030.0,
                y: 400.0,
            },
            Point {
                x: 1030.0,
                y: 250.0,
            },
        ],
        DrawingAttributes::DEFAULT,
    );
    page.add_outline_polygon(
        &[
            Point {
                x: 1030.0,
                y: 250.0,
            },
            Point {
                x: 1030.0,
                y: 400.0,
            },
            Point {
                x: 1420.0,
                y: 400.0,
            },
            Point {
                x: 1420.0,
                y: 250.0,
            },
        ],
        DrawingAttributes::DEFAULT,
    );

    // create header text labels

    page.add_text(
        "Einsatznummer:",
        190.0,
        340.0,
        40.0,
        DrawingAttributes::DEFAULT,
    );
    page.add_text("Alarmzeit:", 810.0, 340.0, 40.0, DrawingAttributes::DEFAULT);

    // add values:

    page.add_text(
        ems.emergency_number.to_string().as_str(),
        540.0,
        340.0,
        40.0,
        DrawingAttributes::TEXT_BOLD,
    );

    let time_str = ems.alarm_time.format("%d.%m.%y\n%H:%M").to_string(); // NOTE: seconds are not transmitted in the mail
    page.add_multiline_text(time_str, 1060.0, 320.0, DrawingAttributes::TEXT_BOLD);
}

fn create_emergency_xps(ems: &Emergency) -> XPSSingleDocument {
    let mut doc = XPSSingleDocument::new().unwrap();
    let page_id = doc.newPage().unwrap();
    let page = doc.page_at(page_id).unwrap();

    add_emergency_header_section(ems, page);

    // let labels = [
    //     "Stichwort:",
    //     "Einsatzort:",
    //     "sonst.",
    //     "Ortsangaben:",
    //     "AAO:",
    //     "FWPlan-Nr:",
    //     "Meldender:",
    //     "Patient 1:",
    //     "Zielort:",
    //     "Ereignis:",
    // ];
    // let label_offsets = [
    //     520.0, 740.0, 940.0, 990.0, 1090.0, 1200.0, 1300.0, 1410.0, 1610.0, 1810.0,
    // ];

    // for (i, label) in labels.iter().enumerate() {
    //     page.add_text(
    //         label.to_string(),
    //         LABEL_OFFSET,
    //         label_offsets[i],
    //         40.0,
    //         DrawingAttributes::DEFAULT,
    //     );
    // }
    let mut curr_y = 520.0;

    let text = format!("{}\n{}", ems.keyword, ems.code3);
    curr_y = add_optional_property(page, "Stichwort:", Some(text), curr_y);

    curr_y = add_optional_property(page, "Einsatzort:", Some(ems.address_text()), curr_y);

    curr_y = add_optional_ml_property(
        page,
        "sonst.\nOrtsangaben:".to_string(),
        Some(ems.location.clone()),
        curr_y,
    );

    curr_y = add_optional_property(page, "FWPlan-Nr:", ems.fire_department_plan.clone(), curr_y);

    // TODO: meldender?

    curr_y = add_optional_property(page, "Patient 1:", ems.patient_name.clone(), curr_y);

    page.add_horizontal_divider(curr_y);

    curr_y += LINE_HEIGHT;

    if let Some(note) = ems.note.clone() {
        if !note.is_empty() {
            page.add_text(
                "Hinweise",
                150.0,
                curr_y,
                40.0,
                DrawingAttributes::TEXT_BOLD,
            );
        }
        curr_y += LINE_HEIGHT * 1.5;
        curr_y = page.add_multiline_text(note, LABEL_OFFSET, curr_y, DrawingAttributes::TEXT_BOLD);
        page.add_horizontal_divider(curr_y);
        curr_y += LINE_HEIGHT;
    }

    create_unit_table(ems, page, curr_y);

    return doc;
}

fn add_optional_property(page: &mut XPSPage, label: &str, p: Option<String>, y: f32) -> f32 {
    let Some(property) = p else {
        return y;
    };
    if property.is_empty() {
        return y;
    }
    let mut y = y;
    page.add_text(label, LABEL_OFFSET, y, 40.0, DrawingAttributes::DEFAULT);
    y = page.add_multiline_text(property, 500.0, y, DrawingAttributes::TEXT_BOLD);

    return y + LINE_HEIGHT;
}

fn add_optional_ml_property(page: &mut XPSPage, label: String, p: Option<String>, y: f32) -> f32 {
    let Some(property) = p else {
        return y;
    };

    if property.is_empty() {
        return y;
    }
    let mut y = y;
    page.add_multiline_text(label, LABEL_OFFSET, y, DrawingAttributes::DEFAULT);
    y = page.add_multiline_text(property, 500.0, y, DrawingAttributes::TEXT_BOLD);

    return y + LINE_HEIGHT;
}

fn create_unit_table(ems: &Emergency, page: &mut XPSPage, start_y: f32) {
    let mut start_y = start_y;
    // create header:
    page.add_text(
        "Alarmierungen",
        150.0,
        start_y,
        40.0,
        DrawingAttributes::TEXT_BOLD,
    );
    start_y += LINE_HEIGHT * 1.5;

    // create column 1 (radio id):
    let max_len = add_column(
        "Funkrufname",
        ems.unit_alarm_times.iter().map(|u| match &u.unit_id {
            Either::Left(id) => id.to_string(),
            Either::Right(id) => id.clone(),
        }),
        page,
        LABEL_OFFSET,
        start_y,
    );
    let x_offset = CHAR_WIDTH_40 * max_len as f32 + LABEL_OFFSET + 20.0;

    // create column 2 (wache):
    let max_len = add_column(
        "Wache",
        ems.unit_alarm_times.iter().map(|u| u.station.clone()),
        page,
        x_offset,
        start_y,
    );
    let x_offset = x_offset + CHAR_WIDTH_40 * max_len as f32 + 20.0;

    // create column 3 (alarm time):
    let max_len = add_column(
        "Alarmzeit",
        ems.unit_alarm_times.iter().map(|u| u.alarm_time.clone()),
        page,
        x_offset,
        start_y,
    );
}

fn add_column<'a, I>(label: &str, values: I, page: &mut XPSPage, x: f32, y: f32) -> usize
where
    I: Iterator<Item = String>,
{
    page.add_text(label, x, y, 40.0, DrawingAttributes::TEXT_BOLD);
    let mut y = y + LINE_HEIGHT;
    let mut max_len = 0;

    for value in values {
        page.add_text(&value.as_str(), x, y, 40.0, DrawingAttributes::DEFAULT);
        max_len = max(max_len, value.len());
        y += LINE_HEIGHT;
    }
    max_len
}
