use std::{cmp::max, fs, io::BufWriter, path::Path, rc::Rc};

use log::{error, trace};

use crate::{
    config::Config,
    models::{either::Either, emergency::Emergency},
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

pub fn print_emergency(ems: Emergency, config: &Config) {
    let mut doc = PDFDocument::new();
    create_emergency_xps(&ems, &mut doc);
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
    printer.print(1, config);
}

fn add_emergency_header_section(ems: &Emergency, page: &mut dyn PageBuilder) {
    // create header blocks

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
}

fn create_emergency_xps(ems: &Emergency, doc: &mut dyn DocumentBuilder) {
    let page_id = doc.new_page().unwrap();
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
    let mut curr_y = 52.0;

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

    curr_y += points_to_mm!(LINE_HEIGHT) * 1.5;

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
        curr_y += points_to_mm!(LINE_HEIGHT) * 1.5;
    }

    create_unit_table(ems, page, curr_y);
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

    return y + points_to_mm!(LINE_HEIGHT);
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
    page.add_multiline_text(
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

    return y + points_to_mm!(LINE_HEIGHT);
}

fn create_unit_table(ems: &Emergency, page: &mut dyn PageBuilder, start_y: f32) {
    let mut start_y = start_y;
    // create header:
    page.add_text(
        "Alarmierungen",
        15.0,
        start_y,
        DEFAULT_FONT_SIZE,
        DrawingAttributes::TEXT_BOLD,
    );
    start_y += LINE_HEIGHT;

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
    let x_offset = CHAR_WIDTH_40 * max_len as f32 + LABEL_OFFSET + 2.0;

    // create column 2 (wache):
    let max_len = add_column(
        "Wache",
        ems.unit_alarm_times.iter().map(|u| u.station.clone()),
        page,
        x_offset,
        start_y,
    );
    let x_offset = x_offset + CHAR_WIDTH_40 * max_len as f32 + 2.0;

    // create column 3 (alarm time):
    let _max_len = add_column(
        "Alarmzeit",
        ems.unit_alarm_times.iter().map(|u| u.alarm_time.clone()),
        page,
        x_offset,
        start_y,
    );
}

fn add_column<'a, I>(label: &str, values: I, page: &mut dyn PageBuilder, x: f32, y: f32) -> usize
where
    I: Iterator<Item = String>,
{
    page.add_text(label, x, y, DEFAULT_FONT_SIZE, DrawingAttributes::TEXT_BOLD);
    let mut y = y + LINE_HEIGHT;
    let mut max_len = label.len() + 3; //  3 looks good :), with 0 all labels would be written as if they were a single long label

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
