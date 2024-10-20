use std::{cell::RefCell, io::Cursor, rc::Weak};

use printpdf::{
    image_crate::codecs, Color, Image, ImageTransform, IndirectFontRef, Line, Mm,
    PdfDocumentReference, PdfLayer, PdfLayerIndex, PdfLayerReference, PdfPageIndex, Rgb,
};

use crate::{
    font_size, line_thickness, points_to_mm,
    printing::document::{DrawingAttributes, PageBuilder, Point},
    text_line_height,
};

#[derive(Clone)]
pub struct PDFPage {
    pub(super) nr: PdfPageIndex,
    pub(super) document: Weak<RefCell<PdfDocumentReference>>,
    pub(super) layer: PdfLayerIndex,
    dimensions: (f32, f32),
    fonts: Vec<IndirectFontRef>,
}

pub const MARGIN_HORIZONTAL: f32 = 15.0;
pub const MARGIN_VERTICAL: f32 = 20.0;
/// the height of one line in pts
/// use the [point_to_mm!()] macro to convert to mm

const FONT_MEDIUM: &'static [u8] = include_bytes!("../../../resources/fonts/PTSerif-Regular.ttf");
const FONT_BOLD: &'static [u8] = include_bytes!("../../../resources/fonts/PTSerif-Bold.ttf");

impl PDFPage {
    pub fn new(
        index: PdfPageIndex,
        doc: Weak<RefCell<PdfDocumentReference>>,
        layer1: PdfLayerIndex,
        dimens: (f32, f32),
    ) -> Self {
        let page = Self {
            nr: index,
            document: doc,
            layer: layer1,
            dimensions: dimens,
            fonts: vec![],
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

    // helpers:
    fn get_font(&mut self, bold: bool) -> IndirectFontRef {
        if self.fonts.len() < 2 {
            let doc = self.document.upgrade().unwrap();
            let doc = doc.borrow();
            self.fonts.push(doc.add_external_font(FONT_MEDIUM).unwrap());
            self.fonts.push(doc.add_external_font(FONT_BOLD).unwrap());
        }
        return self.fonts[if bold { 1 } else { 0 }].clone();
    }

    fn get_current_layer(&mut self) -> PdfLayerReference {
        return self
            .document
            .upgrade()
            .unwrap()
            .borrow()
            .get_page(self.nr)
            .get_layer(self.layer);
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
        let points = points
            .iter()
            .map(|p| {
                (
                    // Points y needs to be inverted, because the printpdf crate uses the bottom left corner as origin not the top left.
                    printpdf::Point::new(Mm(p.x.into()), Mm((height - p.y).into())),
                    false,
                )
            })
            .collect();
        let mut line = Line {
            points: points,
            is_closed: true,
        };
        let layer = self.get_current_layer();

        layer.set_outline_thickness(line_thickness!(attributes));
        layer.add_line(line);
    }

    fn add_text(&mut self, text: &str, x: f32, y: f32, attributes: DrawingAttributes) {
        let doc = self.document.upgrade().unwrap();
        let doc = doc.borrow();

        let font = self.get_font(attributes.text_bold);
        let layer = doc.get_page(self.nr).get_layer(self.layer);

        layer.use_text(
            text,
            font_size!(attributes),
            Mm(x),
            Mm(self.dimensions.1 - y),
            &font,
        );
    }

    fn max_lines_before_overflow(&self, y: f32, attrs: DrawingAttributes) -> usize {
        let height = self.get_dimnensions().1 - MARGIN_VERTICAL as f32;
        let mut curr_y = y;
        let mut count = 0;
        while curr_y + points_to_mm!(text_line_height!(attrs)) < height {
            curr_y += points_to_mm!(text_line_height!(attrs)) * 1.5;
            count += 1;
        }
        return count;
    }

    fn add_multiline_text(
        &mut self,
        text: String,
        x: f32,
        y: f32,
        attributes: DrawingAttributes,
    ) -> f32 {
        let layer = self
            .document
            .upgrade()
            .unwrap()
            .borrow()
            .get_page(self.nr)
            .get_layer(self.layer);

        let font = self.get_font(attributes.text_bold);

        layer.begin_text_section();

        let font_size = font_size!(attributes);
        layer.set_font(&font, font_size);
        layer.set_text_cursor(Mm(x), Mm(self.dimensions.1 - y));
        layer.set_line_height(font_size + 1.0);

        let mut curr_y = y;
        let lines = text.split("\n");
        for line in lines {
            layer.write_text(line, &font);
            layer.add_line_break();
            curr_y += points_to_mm!(text_line_height!(attributes));
        }

        layer.end_text_section();

        return curr_y;
    }

    fn will_multiline_overflow(
        &self,
        _line_count: usize,
        _y: f32,
        _attrs: DrawingAttributes,
    ) -> bool {
        // TODO: implement
        return true;
    }

    fn add_img(&mut self, content: &[u8], x: f32, y: f32, width: f32, height: f32) {
        let doc = self.document.upgrade().unwrap();
        let doc = doc.borrow();

        let layer = doc.get_page(self.nr).get_layer(self.layer);
        let cursor = Cursor::new(content);
        let image = Image::try_from(codecs::bmp::BmpDecoder::new(cursor).unwrap()).unwrap();

        let (original_w, original_h) = (image.image.width.0, image.image.height.0);

        image.add_to_layer(
            layer,
            ImageTransform {
                rotate: None,
                translate_x: Some(Mm(x)),
                translate_y: Some(Mm(self.dimensions.1 - y)),
                scale_x: Some(width / original_w as f32),
                scale_y: Some(height / original_h as f32),
                ..Default::default()
            },
        );
    }
}
