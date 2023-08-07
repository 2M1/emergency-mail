use std::sync::Arc;

use log::{error, warn};
use windows::{
    core::{Error, HSTRING},
    Win32::{
        Foundation::{FALSE, TRUE},
        Storage::Xps::{
            IXpsOMFontResource, IXpsOMObjectFactory, IXpsOMPage, IXpsOMSolidColorBrush, XPS_POINT,
            XPS_SEGMENT_TYPE_LINE, XPS_SIZE, XPS_STYLE_SIMULATION_BOLD,
        },
    },
};

use crate::printing::document::{DrawingAttributes, PageBuilder, Point};

use super::helper::XPSHelper;

pub const PAGE_MARGIN_A4_DEFAULT: f32 = 150.0; // 1.5cm
pub const PAGE_SIZE_A4: XPS_SIZE = XPS_SIZE {
    width: 2100.0,  // 21cm
    height: 2970.0, // 29.7cm
};
pub const LINE_HEIGHT: f32 = 50.0;
const COORDINATE_MUKTIPLIER: f32 = 10.0; // because xps is not in mm, but 1/10 mm...

pub struct XPSPage {
    pub(super) factory: Arc<IXpsOMObjectFactory>,
    pub(super) page: IXpsOMPage,
    pub margin: f32,
    size: XPS_SIZE,
    font: Arc<IXpsOMFontResource>,
}

impl XPSPage {
    pub fn new(factory: Arc<IXpsOMObjectFactory>, page: IXpsOMPage) -> Result<XPSPage, ()> {
        let size = unsafe { page.GetPageDimensions() };
        let Ok(size) = size else {
            error!("couldn't get page dimensions: {:?}", size.unwrap_err());
            return Err(());
        };

        return Ok(XPSPage {
            factory: factory.clone(),
            page: page,
            margin: PAGE_MARGIN_A4_DEFAULT,
            size: size,
            font: Arc::new(XPSHelper::load_font(
                Arc::clone(&factory).as_ref(),
                "Arial",
            )?),
        });
    }

    pub fn should_wrap(&self, y: f32) -> bool {
        return y >= (self.size.height - self.margin);
    }

    fn _get_glyph_run(
        &self,
        font: &IXpsOMFontResource,
        x: f32,
        y: f32,
        size: f32,
        brush: IXpsOMSolidColorBrush,
    ) -> Result<windows::Win32::Storage::Xps::IXpsOMGlyphs, Error> {
        let x = x * COORDINATE_MUKTIPLIER;
        let y = y * COORDINATE_MUKTIPLIER;

        let glyphs = unsafe { self.factory.CreateGlyphs(font) }?;

        unsafe { glyphs.SetOrigin(&XPS_POINT { x: x, y: y }) }?;
        unsafe { glyphs.SetFontRenderingEmSize(size) }?;
        unsafe { glyphs.SetFillBrushLocal(&brush) }?;

        return Ok(glyphs);
    }
}

impl PageBuilder for XPSPage {
    fn get_dimnensions(&self) -> (f32, f32) {
        return (self.size.width / 10.0, self.size.height / 10.0);
    }

    fn max_lines_before_overflow(
        &self,
        y: f32,
        _font_size: f32,
        _attrs: DrawingAttributes,
    ) -> usize {
        let y = y * COORDINATE_MUKTIPLIER;

        let mut curr_y = y;
        let mut lines = 0;
        while !self.should_wrap(curr_y) {
            curr_y += LINE_HEIGHT;
            lines += 1;
        }
        return lines; // TODO: validate this
    }

    fn will_multiline_overflow(
        &self,
        line_count: usize,
        y: f32,
        _font_size: f32,
        _attrs: DrawingAttributes,
    ) -> bool {
        let y = y * COORDINATE_MUKTIPLIER;

        let mut curr_y = y;
        for _ in 0..line_count {
            curr_y += LINE_HEIGHT;
        }
        return self.should_wrap(curr_y);
    }

    fn add_text(&mut self, text: &str, x: f32, y: f32, size: f32, attributes: DrawingAttributes) {
        let x = x * COORDINATE_MUKTIPLIER;
        let y = y * COORDINATE_MUKTIPLIER;

        debug_assert!(x >= self.margin);
        debug_assert!(x <= (self.size.width - self.margin));
        debug_assert!(y >= self.margin);
        debug_assert!(y <= (self.size.height - self.margin));

        let page = &self.page;

        let brush = XPSHelper::create_colour_brush(&self.factory, 0, 0, 0);
        let Ok(brush) = brush else {
            error!("couldn't create brush");
            return;
        };

        let glyphs = self._get_glyph_run(Arc::clone(&self.font).as_ref(), x, y, size, brush);
        let Ok(glyphs) = glyphs else {
            error!("couldn't create glyphs run: {:?}", glyphs.unwrap_err());
            return;
        };
        if attributes.text_bold {
            let bold_res = unsafe { glyphs.SetStyleSimulations(XPS_STYLE_SIMULATION_BOLD) };
            if let Err(_e) = bold_res {
                warn!("failed to create bold text, continuing with normal font!");
            }
        }

        let glyphs_editor = unsafe { glyphs.GetGlyphsEditor() };
        let Ok(glyphs_editor) = glyphs_editor else {
            error!("couldn't get glyphs editor: {:?}", glyphs_editor.unwrap_err());
            return;
        };

        let text_res = unsafe { glyphs_editor.SetUnicodeString(&HSTRING::from(text)) };
        let Ok(_) = text_res else {
            error!("couldn't set text: {:?}", text_res.unwrap_err());
            return;
        };

        let apply_res = unsafe { glyphs_editor.ApplyEdits() };
        let Ok(_) = apply_res else {
            error!("couldn't apply edits: {:?}", apply_res.unwrap_err());
            return;
        };

        let page_add_res = unsafe { page.GetVisuals().unwrap().Append(&glyphs) };
        let Ok(_) = page_add_res else {
            error!("couldn't add glyphs to page: {:?}", page_add_res.unwrap_err());
            return;
        };
    }

    fn add_outline_polygon(&mut self, points: &[Point], attributes: DrawingAttributes) {
        assert!(points.len() >= 2);

        let start = &points[0];
        let figure = unsafe {
            self.factory.CreateGeometryFigure(&XPS_POINT {
                x: start.x * COORDINATE_MUKTIPLIER,
                y: start.y * COORDINATE_MUKTIPLIER,
            })
        };

        let Ok(figure) = figure else {
            error!("couldn't create geometry figure: {:?}", figure.unwrap_err());
            return;
        };

        let segment_types = vec![XPS_SEGMENT_TYPE_LINE; points.len()];
        let mut segment_data: Vec<f32> = Vec::with_capacity(points.len() * 2);

        for point in points {
            let point = XPS_POINT {
                x: point.x * COORDINATE_MUKTIPLIER,
                y: point.y * COORDINATE_MUKTIPLIER,
            };
            debug_assert!(point.x >= self.margin);
            debug_assert!(point.y >= self.margin);
            debug_assert!(point.x <= (self.size.width - self.margin));
            debug_assert!(point.y <= (self.size.height - self.margin));

            segment_data.push(point.x);
            segment_data.push(point.y);
        }

        let draw_lines = vec![TRUE; points.len() - 1];

        let seg_res = unsafe {
            figure.SetSegments(
                segment_types.len() as u32,
                segment_data.len() as u32,
                segment_types.as_ptr(),
                segment_data.as_ptr(),
                draw_lines.as_ptr(),
            )
        };
        let Ok(_) = seg_res else {
            error!("couldn't set segments: {:?}", seg_res.unwrap_err());
            return;
        };
        let is_closed_res = unsafe { figure.SetIsClosed(TRUE) };
        let Ok(_) = is_closed_res else {
            error!("couldn't set is_closed: {:?}", is_closed_res.unwrap_err());
            return;
        };

        let filled_res = unsafe { figure.SetIsFilled(FALSE) };
        let Ok(_) = filled_res else {
            error!("couldn't set is_filled: {:?}", filled_res.unwrap_err());
            return;
        };

        let geometry = unsafe { self.factory.CreateGeometry() };
        let Ok(geometry) = geometry else {
            error!("couldn't create geometry: {:?}", geometry.unwrap_err());
            return;
        };

        let add_res = unsafe { geometry.GetFigures().unwrap().Append(&figure) };
        let Ok(_) = add_res else {
            error!("couldn't add figure to geometry: {:?}", add_res.unwrap_err());
            return;
        };

        let path = unsafe { self.factory.CreatePath() };
        let Ok(path) = path else {
            error!("couldn't create path: {:?}", path.unwrap_err());
            return;
        };

        let geometry_res = unsafe { path.SetGeometryLocal(&geometry) };
        let Ok(_) = geometry_res else {
            error!("couldn't set geometry: {:?}", geometry_res.unwrap_err());
            return;
        };

        unsafe { path.SetStrokeThickness(attributes.line_thickness).unwrap() };
        let brush = XPSHelper::create_colour_brush(&self.factory, 0, 0, 0).unwrap();
        let brush_res = unsafe { path.SetStrokeBrushLocal(&brush) };
        let Ok(_) = brush_res else {
            error!("couldn't set brush: {:?}", brush_res.unwrap_err());
            return;
        };

        let page_add_res = unsafe { self.page.GetVisuals().unwrap().Append(&path) };
        let Ok(_) = page_add_res else {
            error!("couldn't add path to page: {:?}", page_add_res.unwrap_err());
            return;
        };
    }

    /// Adds multiple lines of text seperated by \n to the page.
    ///
    /// Returns the lowest y coordinate of the text.
    fn add_multiline_text(
        &mut self,
        text: String,
        x: f32,
        start_y: f32,
        font_size: f32,
        attributes: DrawingAttributes,
    ) -> f32 {
        let x = x * COORDINATE_MUKTIPLIER;
        let start_y = start_y * COORDINATE_MUKTIPLIER;

        debug_assert!(x >= self.margin);
        debug_assert!(x <= (self.size.width - self.margin));
        debug_assert!(start_y >= self.margin);
        debug_assert!(start_y <= (self.size.height - self.margin));

        let lines = text.split("\n");

        let mut curr_y = start_y;
        for line in lines {
            self.add_text(line, x / 10.0, curr_y / 10.0, font_size, attributes);
            curr_y += LINE_HEIGHT;
        }

        return curr_y;
    }

    fn add_horizontal_divider(&mut self, y: f32) {
        let y = y * COORDINATE_MUKTIPLIER;

        debug_assert!(y >= self.margin);
        debug_assert!(y <= (self.size.height - self.margin));

        let points = vec![
            Point {
                x: self.margin,
                y: y,
            },
            Point {
                x: self.size.width - self.margin,
                y: y,
            },
        ];
        self.add_outline_polygon(&points, DrawingAttributes::DEFAULT);
    }

    fn add_img(&mut self, content: &[u8], x: f32, y: f32, width: usize, height: usize) {
        unimplemented!("adding images is not yet supported for xps printing");
    }
}
