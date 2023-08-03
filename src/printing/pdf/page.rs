use std::{cell::RefCell, fs::File, rc::Weak};

use printpdf::{
    Color, DirectFontRef, Line, Mm, PdfDocumentReference, PdfLayerIndex, PdfPageIndex, Rgb,
};

use crate::printing::document::{DrawingAttributes, PageBuilder, Point};

#[derive(Clone)]
pub struct PDFPage {
    pub(super) nr: PdfPageIndex,
    pub(super) document: Weak<RefCell<PdfDocumentReference>>,
    pub(super) layer: PdfLayerIndex,
    dimensions: (f64, f64),
}

pub const MARGIN_HORIZONTAL: f64 = 15.0;
pub const LINE_HEIGHT: f32 = 13.0;

impl PDFPage {
    pub fn new(
        index: PdfPageIndex,
        doc: Weak<RefCell<PdfDocumentReference>>,
        layer1: PdfLayerIndex,
        dimens: (f64, f64),
    ) -> Self {
        let page = Self {
            nr: index,
            document: doc,
            layer: layer1,
            dimensions: dimens,
        };

        let layer = page
            .document
            .upgrade()
            .unwrap()
            .borrow()
            .get_page(page.nr)
            .get_layer(page.layer);

        layer.set_fill_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));
        layer.set_outline_color(Color::Rgb(Rgb::new(0.0, 0.0, 0.0, None)));

        return page;
    }
}

impl PageBuilder for PDFPage {
    fn get_dimnensions(&self) -> (f32, f32) {
        return (self.dimensions.0 as f32, self.dimensions.1 as f32);
    }

    fn add_horizontal_divider(&mut self, y: f32) {
        debug_assert!(y >= 0.0);
        debug_assert!(y <= self.dimensions.1 as f32);

        // NOTE: use direct y as input here, because the coordinate system transformation is done in the add_outline_polygon function
        self.add_outline_polygon(
            &vec![
                Point {
                    x: MARGIN_HORIZONTAL as f32,
                    y: y,
                },
                Point {
                    x: self.dimensions.0 as f32 - MARGIN_HORIZONTAL as f32,
                    y: y,
                },
            ],
            DrawingAttributes::DEFAULT,
        )
    }

    fn add_outline_polygon(&mut self, points: &[Point], attributes: DrawingAttributes) {
        let height = self.get_dimnensions().1;
        let points = points.iter().map(|p| {
            (
                // Points y needs to be inverted, because the printpdf crate uses the bottom left corner as origin not the top left.
                printpdf::Point::new(Mm(p.x as f64), Mm((height - p.y) as f64)),
                false,
            )
        });
        let mut line = Line::from_iter(points);
        line.set_closed(true);
        line.set_stroke(true);
        line.set_fill(false);

        let layer = self
            .document
            .upgrade()
            .unwrap()
            .borrow()
            .get_page(self.nr)
            .get_layer(self.layer);

        layer.set_outline_thickness(attributes.line_thickness as f64);
        layer.add_shape(line);
    }

    fn add_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        attributes: DrawingAttributes,
    ) {
        let doc = self.document.upgrade().unwrap();
        let doc = doc.borrow();
        let font_path = if attributes.text_bold {
            "C:\\Windows\\Fonts\\ARIALNB.ttf"
        } else {
            "C:\\Windows\\Fonts\\ARIAL.ttf"
        };
        let font = doc
            .add_external_font(File::open(font_path).unwrap())
            .unwrap();
        let layer = doc.get_page(self.nr).get_layer(self.layer);

        layer.use_text(
            text,
            font_size as f64,
            Mm(x as f64),
            Mm(self.dimensions.1 - y as f64),
            &font,
        );
    }

    fn max_lines_before_overflow(
        &self,
        _y: f32,
        _font_size: f32,
        _attrs: DrawingAttributes,
    ) -> usize {
        // TODO: implement
        return 0;
    }

    fn add_multiline_text(
        &mut self,
        _text: String,
        _x: f32,
        y: f32,
        _attributes: DrawingAttributes,
    ) -> f32 {
        // TODO: implement
        return y;
    }

    fn will_multiline_overflow(
        &self,
        _line_count: usize,
        _y: f32,
        _font_size: f32,
        _attrs: DrawingAttributes,
    ) -> bool {
        // TODO: implement
        return true;
    }
}
