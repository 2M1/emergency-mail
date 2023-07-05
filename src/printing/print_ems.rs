use crate::models::emergency::Emergency;

use super::{print_xps::print_test, xps_document::XPSSingleDocument, xps_page::Point};

pub fn print_emergency(ems: Emergency) {
    let doc = create_emergency_xps(ems);
    print_test(doc);
}

fn create_emergency_xps(ems: Emergency) -> XPSSingleDocument {
    let mut doc = XPSSingleDocument::new().unwrap();
    let page_id = doc.newPage().unwrap();
    doc.page_at(page_id)
        .unwrap()
        .add_text("Hello world".to_string(), 10.0, 10.0);
    doc.page_at(page_id).unwrap().add_outline_polygon(&[
        Point { x: 30.0, y: 30.0 },
        Point { x: 30.0, y: 60.0 },
        Point { x: 60.0, y: 60.0 },
        Point { x: 60.0, y: 30.0 },
    ]);
    let _ = doc.newPage().unwrap();
    return doc;
}
