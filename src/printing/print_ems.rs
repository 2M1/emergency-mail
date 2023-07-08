use crate::models::emergency::Emergency;

use super::{print_xps::print_test, xps_document::XPSSingleDocument, xps_page::Point};

const LABEL_OFFSET: f32 = 180.0;
const SECTION_OFFSET: f32 = 150.0;

pub fn print_emergency(ems: Emergency) {
    let doc = create_emergency_xps(ems);
    // print_test(doc);
    doc.safe();
}

fn create_emergency_xps(ems: Emergency) -> XPSSingleDocument {
    let mut doc = XPSSingleDocument::new().unwrap();
    let page_id = doc.newPage().unwrap();
    let page = doc.page_at(page_id).unwrap();

    // create header blocks

    page.add_outline_polygon(&[
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
    ]);
    page.add_outline_polygon(&[
        Point { x: 500.0, y: 250.0 },
        Point { x: 500.0, y: 400.0 },
        Point { x: 780.0, y: 400.0 },
        Point { x: 780.0, y: 250.0 },
    ]);
    page.add_outline_polygon(&[
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
    ]);
    page.add_outline_polygon(&[
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
    ]);

    // create header text labels

    page.add_text("Einsatznummer:".to_string(), 190.0, 340.0, 40.0);
    page.add_text("Alarmzeit:".to_string(), 810.0, 340.0, 40.0);

    // create divider line
    page.add_outline_polygon(&[
        Point {
            x: SECTION_OFFSET,
            y: 1870.0,
        },
        Point {
            x: SECTION_OFFSET,
            y: 1870.0,
        },
        Point {
            x: 1950.0,
            y: 1870.0,
        },
        Point {
            x: 1950.0,
            y: 1870.0,
        },
    ]);

    let labels = [
        "Stichwort:",
        "Einsatzort:",
        "sonst.",
        "Ortsangaben:",
        "AAO:",
        "FWPlan-Nr:",
        "Meldender:",
        "Patient 1:",
        "Zielort:",
        "Ereignis:",
    ];
    let label_offsets = [
        520.0, 740.0, 940.0, 990.0, 1090.0, 1200.0, 1300.0, 1410.0, 1610.0, 1810.0,
    ];

    for (i, label) in labels.iter().enumerate() {
        page.add_text(label.to_string(), LABEL_OFFSET, label_offsets[i], 40.0);
    }

    return doc;
}
