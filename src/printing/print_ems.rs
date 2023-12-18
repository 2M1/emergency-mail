use chrono::Local;
use std::path::PathBuf;
use std::{
    cmp::{max, min},
    fs,
    io::BufWriter,
    path::Path,
    rc::Rc,
};

use log::{error, info, trace};

use crate::{
    config::Config,
    models::{either::Either, emergency::Emergency},
    points_to_mm,
    printing::{
        document::{DocumentBuildingError, Printable},
        pdf::{document::PDFDocument, print_pdf::PDFFilePrinter},
    },
    text_line_height,
};

use super::document::{DocumentBuilder, DrawingAttributes, PageBuilder, Point, Size};

impl DrawingAttributes {
    pub const OUTLINE_POLY: Self = Self {
        text_bold: false,
        size: Size {
            line_thickness: 1.0,
        },
    };

    pub const LABEL: Self = Self {
        text_bold: true,
        size: Size {
            font_size: DEFAULT_FONT_SIZE,
        },
    };

    pub const FIELD_VALUE: Self = Self {
        text_bold: false,
        size: Size {
            font_size: DEFAULT_FONT_SIZE,
        },
    };

    pub const HIGHLIGHTED_ENTRY: Self = Self {
        text_bold: true,
        size: Size {
            font_size: DEFAULT_FONT_SIZE,
        },
    };
}

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
    create_emergency_doc(&ems, &mut doc, config);

    let mut ems_dir: PathBuf = if config.pdf_save_path.is_some() {
        Path::new(config.pdf_save_path.as_ref().unwrap().as_str()).to_path_buf()
    } else {
        let mut tmp = std::env::temp_dir();
        tmp.push(Path::new("emergency_mail\\"));
        tmp
    };

    let res = fs::create_dir_all(&ems_dir);
    if let Err(e) = res {
        error!("couldn't create temp dir: {}", e);
        return;
    }

    ems_dir.push(format!(
        "{}_{}.pdf",
        Local::now().format("%Y-%m-%d_%H-%M-%S"),
        // using - since windows does not allow : in file names
        // using current time since alarm time could be duplicated when multiple mails are send (e.g. resend)
        ems.keyword.replace(':', "-"), // due to windows, see above.
    ));
    if cfg!(debug_assertions) || (config.printing.disabled() && config.pdf_save_path.is_none()) {
        ems_dir = Path::new("test.pdf").to_path_buf();
    }

    trace!("saving to: {:?}", &ems_dir);
    let docref = doc.document;
    let docref = Rc::try_unwrap(docref)
        .map_err(|_| DocumentBuildingError::Error("couldn't unwrap document".to_string()))
        .unwrap()
        .into_inner();
    let file = fs::File::create(&ems_dir).unwrap();
    let mut writer = BufWriter::new(file);
    docref.save(&mut writer).unwrap();

    let printer = PDFFilePrinter::new(ems_dir.as_path());
    printer.print(count_copies(&ems, config), config);
}

pub(super) fn count_units_from_configured_amt(ems: &Emergency, config: &Config) -> usize {
    let mut count = 0;
    for unit in &ems.dispatched_units {
        let Either::Left(unit) = unit else {
            continue;
        };
        // skipp all units, that do not have a standard radio id (Funkkenner)

        if unit.agency == config.printing.amt && unit.county == "PM" && unit.org == "FL" {
            // NOTE: county and org are hardcoded for now!
            count += 1;
        }
    }
    return count;
}

pub(super) fn count_copies(ems: &Emergency, config: &Config) -> usize {
    let mut count = count_units_from_configured_amt(ems, config);
    if let Some(additional_copies) = config.printing.additional_copies {
        count += additional_copies as usize;
    }
    count = max(count, config.printing.min_copies as usize);
    if let Some(max_copies) = config.printing.max_copies {
        count = min(count, max_copies as usize);
    }
    trace!("number of copies: {}", count);
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
        DrawingAttributes::OUTLINE_POLY,
    );
    page.add_outline_polygon(
        &[
            Point { x: 50.0, y: 25.0 },
            Point { x: 50.0, y: 40.0 },
            Point { x: 78.0, y: 40.0 },
            Point { x: 78.0, y: 25.0 },
        ],
        DrawingAttributes::OUTLINE_POLY,
    );
    page.add_outline_polygon(
        &[
            Point { x: 78.0, y: 25.0 },
            Point { x: 78.0, y: 40.0 },
            Point { x: 103.0, y: 40.0 },
            Point { x: 103.0, y: 25.0 },
        ],
        DrawingAttributes::OUTLINE_POLY,
    );
    page.add_outline_polygon(
        &[
            Point { x: 103.0, y: 25.0 },
            Point { x: 103.0, y: 40.0 },
            Point { x: 142.0, y: 40.0 },
            Point { x: 142.0, y: 25.0 },
        ],
        DrawingAttributes::OUTLINE_POLY,
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
        DrawingAttributes::OUTLINE_POLY,
    );

    // create header text labels

    page.add_text("Einsatznummer:", 16.0, 34.0, DrawingAttributes::LABEL);
    page.add_text("Alarmzeit:", 80.0, 34.0, DrawingAttributes::LABEL);

    // add values:

    page.add_text(
        ems.emergency_number.to_string().as_str(),
        52.0,
        34.0,
        DrawingAttributes::FIELD_VALUE,
    );

    let time_str = ems.alarm_time.format("%d.%m.%y\n%H:%M").to_string(); // NOTE: seconds are not transmitted in the mail
    page.add_multiline_text(time_str, 106.0, 32.0, DrawingAttributes::FIELD_VALUE);

    page.add_multiline_text(
        "Feuerwehr\nKleinmachnow".to_string(),
        160.0,
        32.0,
        DrawingAttributes::LABEL,
    );
}

fn create_emergency_doc(ems: &Emergency, doc: &mut dyn DocumentBuilder, config: &Config) {
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

    curr_y += points_to_mm!(text_line_height!(DrawingAttributes::FIELD_VALUE)) * 1.2;

    if let Some(note) = ems.note.clone() {
        if !note.is_empty() {
            page.add_text("Hinweise", 15.0, curr_y, DrawingAttributes::LABEL);
        }
        curr_y += points_to_mm!(text_line_height!(DrawingAttributes::FIELD_VALUE)) * 1.5;
        curr_y =
            page.add_multiline_text(note, LABEL_OFFSET, curr_y, DrawingAttributes::FIELD_VALUE);
        page.add_horizontal_divider(curr_y);
        curr_y += points_to_mm!(text_line_height!(DrawingAttributes::FIELD_VALUE)) * 1.2;
    }

    let mut offsets = AlarmTableOffsets {
        station: 0.0,
        time: 0.0,
    };
    let home_count = count_units_from_configured_amt(ems, config);
    println!("homecount: {}", home_count);
    let remaining = create_unit_table(ems, page, curr_y, &mut offsets, home_count);
    let printed = ems.unit_alarm_times.len() - remaining;

    if remaining > 0 {
        info!("creating second page");
        let page_id = doc.new_page().unwrap();
        let page = doc.page_at(page_id).unwrap();
        let curr_y = 25.0;
        // assumes that all remaining units fit on one page :):
        let page_units = &ems.unit_alarm_times[ems.unit_alarm_times.len() - remaining..];
        let remaining_home_count = if home_count > printed {
            home_count - printed
        } else {
            0
        };

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
            remaining_home_count,
        );

        // create column 2 (wache):
        add_column(
            0,
            page_units.iter().map(|u| u.station.clone()),
            page,
            offsets.station,
            curr_y,
            remaining_home_count,
        );

        // create column 3 (alarm time):
        add_column(
            0,
            page_units.iter().map(|u| u.alarm_time.clone()),
            page,
            offsets.time,
            curr_y,
            remaining_home_count,
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
    page.add_text(label, LABEL_OFFSET, y, DrawingAttributes::LABEL);
    y = page.add_multiline_text(property, 50.0, y, DrawingAttributes::FIELD_VALUE);

    return y + points_to_mm!(text_line_height!(DrawingAttributes::FIELD_VALUE)) * 1.2;
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
    let h_label = page.add_multiline_text(label, LABEL_OFFSET, y, DrawingAttributes::LABEL);
    y = page.add_multiline_text(property, 50.0, y, DrawingAttributes::FIELD_VALUE);

    return y.max(h_label) + 5.0;
}

fn create_unit_table(
    ems: &Emergency,
    page: &mut dyn PageBuilder,
    start_y: f32,
    offsets: &mut AlarmTableOffsets,
    home_count: usize,
) -> usize {
    let mut start_y = start_y;
    // create header:
    page.add_text("Alarmierungen", 15.0, start_y, DrawingAttributes::LABEL);
    start_y += points_to_mm!(text_line_height!(DrawingAttributes::LABEL)) * 2.0;

    // calculate the number of items that fit on the page (excluding the header: -1):
    let max_items = page.max_lines_before_overflow(start_y, DrawingAttributes::FIELD_VALUE) - 1;
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
        home_count,
    );
    offsets.station = CHAR_WIDTH_40 * max_len as f32 + LABEL_OFFSET + 8.0;

    // create column 2 (wache):
    let max_len = add_start_column(
        page,
        "Wache",
        offsets.station,
        start_y,
        page_units.iter().map(|u| u.station.clone()),
        home_count,
    );
    offsets.time = offsets.station + CHAR_WIDTH_40 * max_len as f32 + 8.0;

    // create column 3 (alarm time):
    add_start_column(
        page,
        "Alarmzeit",
        offsets.time,
        start_y,
        page_units.iter().map(|u| u.alarm_time.clone()),
        home_count,
    );

    return max(0, ems.unit_alarm_times.len() - max_items);
}

fn add_start_column<I>(
    page: &mut dyn PageBuilder,
    label: &str,
    x_offset: f32,
    start_y: f32,
    page_units: I,
    bold_count: usize,
) -> usize
where
    I: Iterator<Item = String>,
{
    page.add_text(label, x_offset, start_y, DrawingAttributes::LABEL);
    return add_column(label.len(), page_units, page, x_offset, start_y, bold_count);
}

fn add_column<'a, I>(
    label_len: usize,
    values: I,
    page: &mut dyn PageBuilder,
    x: f32,
    y: f32,
    bold_count: usize,
) -> usize
where
    I: Iterator<Item = String>,
{
    let mut y = y + points_to_mm!(text_line_height!(DrawingAttributes::FIELD_VALUE)) * 1.5;
    let mut max_len = label_len + 3; //  3 looks good :), with 0 all labels would be written as if they were a single long label
    let mut bold_remaining = bold_count;

    for value in values {
        page.add_text(
            &value.as_str(),
            x,
            y,
            if bold_remaining > 0 {
                bold_remaining -= 1;
                DrawingAttributes::HIGHLIGHTED_ENTRY
            } else {
                DrawingAttributes::FIELD_VALUE
            },
        );
        max_len = max(max_len, value.len());
        y += points_to_mm!(text_line_height!(DrawingAttributes::FIELD_VALUE)) * 1.5;
    }
    max_len
}
