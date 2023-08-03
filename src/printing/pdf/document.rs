use std::{cell::RefCell, error::Error, fs::File, io::BufWriter, path::Path, rc::Rc, sync::Arc};

use printpdf::{Mm, PdfDocument, PdfDocumentReference, PdfPage};

use crate::printing::document::{self, DocumentBuilder, DocumentBuildingError, Saveable};

use super::page::PDFPage;

#[derive(Clone)]
pub struct PDFDocument {
    pub(crate) document: Rc<RefCell<PdfDocumentReference>>,
    pages: Vec<PDFPage>,
    first: bool,
}

const A4_WIDTH: f64 = 210.0;
const A4_HEIGHT: f64 = 297.0;

impl PDFDocument {
    pub fn new() -> Self {
        let (document, p1, l1) = PdfDocument::new(
            "printpdf paginated example",
            Mm(A4_WIDTH),
            Mm(A4_HEIGHT),
            "Layer 1",
        );

        let document = Rc::new(RefCell::new(document));

        Self {
            document: document.clone(),
            pages: vec![PDFPage::new(
                p1,
                Rc::downgrade(&document),
                l1,
                (A4_WIDTH, A4_HEIGHT),
            )],
            first: true,
        }
    }
}

impl DocumentBuilder for PDFDocument {
    fn begin(&mut self) -> Result<(), document::DocumentBuildingError> {
        Ok(())
    }

    fn new_page(&mut self) -> Result<usize, document::DocumentBuildingError> {
        if self.first {
            // the printpdf crate creates a page by default, so we need to skip the first call
            self.first = false;
            return Ok(0);
        }

        let (page, layer) =
            self.document
                .borrow_mut()
                .add_page(Mm(A4_WIDTH), Mm(A4_HEIGHT), "Layer 1");

        self.pages.push(PDFPage::new(
            page,
            Rc::downgrade(&self.document),
            layer,
            (A4_WIDTH, A4_HEIGHT),
        ));

        return Ok(self.pages.len() - 1);
    }

    fn page_at(&mut self, index: usize) -> Option<&mut dyn document::PageBuilder> {
        return self
            .pages
            .get_mut(index)
            .map(|page| page as &mut dyn document::PageBuilder);
    }
}
