use std::{
    cmp::{max, min},
    fs,
    io::BufWriter,
    path::Path,
    rc::Rc,
};

use chrono::Offset;
use log::{error, info, trace};

use crate::{
    config::Config,
    models::{either::Either, emergency::Emergency, unit_alarm_time::UnitAlarmTime},
    points_to_mm,
    printing::{
        document::{DocumentBuildingError, Printable},
        pdf::{document::PDFDocument, print_pdf::PDFFilePrinter},
    },
};

use super::{
    document::{DocumentBuilder, DrawingAttributes, PageBuilder, Point},
    pdf::page::LINE_HEIGHT,
};

const LABEL_OFFSET: f32 = 18.0;
const SECTION_OFFSET: f32 = 15.0;
const CHAR_WIDTH_40: f32 = 2.5;
const DEFAULT_FONT_SIZE: f32 = 12.0;
const LOGO: &[u8] = include_bytes!("../../resources/img/logo-sw.bmp");

struct AlarmTableOffsets {
    // radioId is always = LABEL_OFFSET
    station: f32,
    time: f32,
}

pub fn print_emergency(ems: Emergency, config: &Config) {
    let mut doc = PDFDocument::new();
    create_emergency_doc(&ems, &mut doc);
    let mut temp_dir = std::env::temp_dir();
    temp_dir.push(Path::new("emergency_mail\\"));

    let res = fs::create_dir_all(&temp_dir);
    if let Err(e) = res {
        error!("couldn't create temp dir: {}", e);
        return;
    }
    temp_dir.push("output.pdf");
    if cfg!(debug_assertions) {
        temp_dir = Path::new("test.pdf").to_path_buf();
    }
    trace!("saving to: {:?}", temp_dir);
    let docref = doc.document;
    let docref = Rc::try_unwrap(docref)
        .map_err(|_| DocumentBuildingError::Error("couldn't unwrap document".to_string()))
        .unwrap()
        .into_inner();
    let file = fs::File::create(&temp_dir).unwrap();
    let mut writer = BufWriter::new(file);
    docref.save(&mut writer).unwrap();

    let printer = PDFFilePrinter::new(&temp_dir);
    printer.print(count_copies(&ems, config), config);
}

pub(super) fn count_copies(ems: &Emergency, config: &Config) -> usize {
    let mut count = 0;
    for unit in &ems.dispatched_units {
        let Either::Left(unit) = unit else { continue };
        // skipp all units, that do not have a standard radio id (Funkkenner)

        if unit.agency == config.printing.amt && unit.county == "PM" && unit.org == "FL" {
            // NOTE: county and ord are hardcoded for now!
            count += 1;
        }
    }

    count = max(count, config.printing.min_copies as usize);
    if let Some(max_copies) = config.printing.max_copies {
        count = min(count, max_copies as usize);
    }
    return count;
}

fn add_emergency_header_section(ems: &Emergency, page: &mut dyn PageBuilder) {
    // create header blocks

    page.add_img(LOGO, 142.0, 41.5, 200, 200);

    page.add_outline_polygon(
        &[
            Point {
                x: SECTION_OFFSET,
                y: 25.0,
            },
            Point {
                x: SECTION_OFFSET,
                y: 40.0,
            },
            Point { x: 50.0, y: 40.0 },
            Point { x: 50.0, y: 25.0 },
        ],
        DrawingAttributes::DEFAULT,
    );
    page.add_outline_polygon(
        &[
            Point { x: 50.0, y: 25.0 },
            Point { x: 50.0, y: 40.0 },
            Point { x: 78.0, y: 40.0 },
            Point { x: 78.0, y: 25.0 },
        ],
        DrawingAttributes::DEFAULT,
    );
    page.add_outline_polygon(
        &[
            Point { x: 78.0, y: 25.0 },
            Point { x: 78.0, y: 40.0 },
            Point { x: 103.0, y: 40.0 },
            Point { x: 103.0, y: 25.0 },
        ],
        DrawingAttributes::DEFAULT,
    );
    page.add_outline_polygon(
        &[
            Point { x: 103.0, y: 25.0 },
            Point { x: 103.0, y: 40.0 },
            Point { x: 142.0, y: 40.0 },
            Point { x: 142.0, y: 25.0 },
        ],
        DrawingAttributes::DEFAULT,
    );

    page.add_outline_polygon(
        &[
            Point { x: 142.0, y: 25.0 },
            Point { x: 142.0, y: 40.0 },
            Point {
                x: page.get_dimnensions().0 - SECTION_OFFSET,
                y: 40.0,
            },
            Point {
                x: page.get_dimnensions().0 - SECTION_OFFSET,
                y: 25.0,
            },
        ],
        DrawingAttributes::DEFAULT,
    );

    // create header text labels

    page.add_text(
        "Einsatznummer:",
        16.0,
        34.0,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::DEFAULT,
    );
    page.add_text(
        "Alarmzeit:",
        81.0,
        34.0,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::DEFAULT,
    );

    // add values:

    page.add_text(
        ems.emergency_number.to_string().as_str(),
        52.0,
        34.0,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::TEXT_BOLD,
    );

    let time_str = ems.alarm_time.format("%d.%m.%y\n%H:%M").to_string(); // NOTE: seconds are not transmitted in the mail
    page.add_multiline_text(
        time_str,
        106.0,
        32.0,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::TEXT_BOLD,
    );

    page.add_multiline_text(
        "Feuerwehr\nKleinmachnow".to_string(),
        160.0,
        32.0,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::TEXT_BOLD,
    );
}

fn create_emergency_doc(ems: &Emergency, doc: &mut dyn DocumentBuilder) {
    let page_id = doc.new_page().unwrap();
    let page = doc.page_at(page_id).unwrap();

    add_emergency_header_section(ems, page);

    let mut curr_y = 52.0;

    let text = format!("{}\n{}", ems.keyword, ems.code3);
    curr_y = add_optional_property(page, "Stichwort:", Some(text), curr_y);

    curr_y = add_optional_property(page, "Einsatzort:", Some(ems.address_text()), curr_y);

    curr_y = add_optional_ml_property(
        page,
        "Objekt:".to_string(),
        ems.get_obj_description(),
        curr_y,
    );

    curr_y = add_optional_ml_property(
        page,
        "sonst.\nOrtsangaben:".to_string(),
        ems.location_addition.clone(),
        curr_y,
    );

    curr_y = add_optional_property(page, "FWPlan-Nr:", ems.fire_department_plan.clone(), curr_y);

    curr_y = add_optional_property(page, "Patient:", ems.get_patient_name(), curr_y);

    page.add_horizontal_divider(curr_y);

    curr_y += points_to_mm!(LINE_HEIGHT) * 1.2;

    if let Some(note) = ems.note.clone() {
        if !note.is_empty() {
            page.add_text(
                "Hinweise",
                15.0,
                curr_y,
                DEFAULT_FONT_SIZE,
                DrawingAttributes::TEXT_BOLD,
            );
        }
        curr_y += points_to_mm!(LINE_HEIGHT) * 1.5;
        curr_y = page.add_multiline_text(
            note,
            LABEL_OFFSET,
            curr_y,
            DEFAULT_FONT_SIZE,
            DrawingAttributes::TEXT_BOLD,
        );
        page.add_horizontal_divider(curr_y);
        curr_y += points_to_mm!(LINE_HEIGHT) * 1.2;
    }

    let mut offsets = AlarmTableOffsets {
        station: 0.0,
        time: 0.0,
    };
    let remaining = create_unit_table(ems, page, curr_y, &mut offsets);

    if remaining > 0 {
        info!("creating second page");
        let page_id = doc.new_page().unwrap();
        let page = doc.page_at(page_id).unwrap();
        let curr_y = 25.0;

        // assumes that all remaining units fit on one page :):
        let page_units = &ems.unit_alarm_times[ems.unit_alarm_times.len() - remaining..];

        // create column 1 (radio id):
        add_column(
            0,
            page_units.iter().map(|u| match &u.unit_id {
                Either::Left(id) => id.to_string(),
                Either::Right(id) => id.clone(),
            }),
            page,
            LABEL_OFFSET,
            curr_y,
        );

        // create column 2 (wache):
        add_column(
            0,
            page_units.iter().map(|u| u.station.clone()),
            page,
            offsets.station,
            curr_y,
        );

        // create column 3 (alarm time):
        add_column(
            0,
            page_units.iter().map(|u| u.alarm_time.clone()),
            page,
            offsets.time,
            curr_y,
        );
    }
}

fn add_optional_property(
    page: &mut dyn PageBuilder,
    label: &str,
    p: Option<String>,
    y: f32,
) -> f32 {
    let Some(property) = p else {
        return y;
    };
    if property.is_empty() {
        return y;
    }
    let mut y = y;
    page.add_text(
        label,
        LABEL_OFFSET,
        y,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::DEFAULT,
    );
    y = page.add_multiline_text(
        property,
        50.0,
        y,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::TEXT_BOLD,
    );

    return y + points_to_mm!(LINE_HEIGHT) * 1.2;
}

fn add_optional_ml_property(
    page: &mut dyn PageBuilder,
    label: String,
    p: Option<String>,
    y: f32,
) -> f32 {
    let Some(property) = p else {
        return y;
    };

    if property.is_empty() {
        return y;
    }
    let mut y = y;
    let h_label = page.add_multiline_text(
        label,
        LABEL_OFFSET,
        y,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::DEFAULT,
    );
    y = page.add_multiline_text(
        property,
        50.0,
        y,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::TEXT_BOLD,
    );

    return y.max(h_label) + 5.0;
}

fn create_unit_table(
    ems: &Emergency,
    page: &mut dyn PageBuilder,
    start_y: f32,
    offsets: &mut AlarmTableOffsets,
) -> usize {
    let mut start_y = start_y;
    // create header:
    page.add_text(
        "Alarmierungen",
        15.0,
        start_y,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::TEXT_BOLD,
    );
    start_y += points_to_mm!(LINE_HEIGHT) * 2.0;

    // calculate the number of items that fit on the page (excluding the header: -1):
    let max_items =
        page.max_lines_before_overflow(start_y, DEFAULT_FONT_SIZE, DrawingAttributes::DEFAULT) - 1;
    let max_items = min(ems.unit_alarm_times.len(), max_items);

    let page_units = &ems.unit_alarm_times[0..max_items];

    // create column 1 (radio id):
    let max_len = add_start_column(
        page,
        "Funkrufname",
        LABEL_OFFSET,
        start_y,
        page_units.iter().map(|u| match &u.unit_id {
            Either::Left(id) => id.to_string(),
            Either::Right(id) => id.clone(),
        }),
    );
    offsets.station = CHAR_WIDTH_40 * max_len as f32 + LABEL_OFFSET + 8.0;

    // create column 2 (wache):
    let max_len = add_start_column(
        page,
        "Wache",
        offsets.station,
        start_y,
        page_units.iter().map(|u| u.station.clone()),
    );
    offsets.time = offsets.station + CHAR_WIDTH_40 * max_len as f32 + 8.0;

    // create column 3 (alarm time):
    add_start_column(
        page,
        "Alarmzeit",
        offsets.time,
        start_y,
        page_units.iter().map(|u| u.alarm_time.clone()),
    );

    return max(0, ems.unit_alarm_times.len() - max_items);
}

fn add_start_column<I>(
    page: &mut dyn PageBuilder,
    label: &str,
    x_offset: f32,
    start_y: f32,
    page_units: I,
) -> usize
where
    I: Iterator<Item = String>,
{
    page.add_text(
        label,
        x_offset,
        start_y,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::TEXT_BOLD,
    );
    return add_column(label.len(), page_units, page, x_offset, start_y);
}

fn add_column<'a, I>(
    label_len: usize,
    values: I,
    page: &mut dyn PageBuilder,
    x: f32,
    y: f32,
) -> usize
where
    I: Iterator<Item = String>,
{
    let mut y = y + points_to_mm!(LINE_HEIGHT) * 1.5;
    let mut max_len = label_len + 3; //  3 looks good :), with 0 all labels would be written as if they were a single long label

    for value in values {
        page.add_text(
            &value.as_str(),
            x,
            y,
            DEFAULT_FONT_SIZE,
            DrawingAttributes::DEFAULT,
        );
        max_len = max(max_len, value.len());
        y += points_to_mm!(LINE_HEIGHT) * 1.5;
    }
    max_len
}
