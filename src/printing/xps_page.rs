use std::sync::Arc;

use log::error;
use windows::{
    core::HSTRING,
    Win32::{
        Foundation::{FALSE, TRUE},
        Storage::Xps::{
            IXpsOMObjectFactory, IXpsOMPage, XPS_POINT, XPS_SEGMENT_TYPE, XPS_SEGMENT_TYPE_LINE,
        },
    },
};

use super::xps_helper::XPSHelper;

pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub struct XPSPage {
    pub(super) factory: Arc<IXpsOMObjectFactory>,
    pub(super) page: IXpsOMPage,
}

impl XPSPage {
    pub fn add_text(&mut self, text: String, x: f32, y: f32) {
        let page = &self.page;

        let font = XPSHelper::load_font(&self.factory, "Arial").unwrap();

        let glyphs = unsafe { self.factory.CreateGlyphs(&font) };
        let Ok(glyphs) = glyphs else {
            error!("couldn't create glyphs: {:?}", glyphs.unwrap_err());
            return;
        };

        let origin_res = unsafe { glyphs.SetOrigin(&XPS_POINT { x: x, y: y }) };
        let Ok(_) = origin_res else {
            error!("couldn't set origin: {:?}", origin_res.unwrap_err());
            return;
        };

        let font_size_res = unsafe { glyphs.SetFontRenderingEmSize(12.0) };
        let Ok(_) = font_size_res else {
            error!("couldn't set font size: {:?}", font_size_res.unwrap_err());
            return;
        };

        let brush = XPSHelper::create_colour_brush(&self.factory, 0, 0, 0).unwrap();
        let brush_res = unsafe { glyphs.SetFillBrushLocal(&brush) };
        let Ok(_) = brush_res else {
            error!("couldn't set brush: {:?}", brush_res.unwrap_err());
            return;
        };

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

    pub fn add_outline_polygon(&mut self, points: &[Point]) {
        assert!(points.len() >= 2);

        let start = &points[0];
        let figure = unsafe {
            self.factory.CreateGeometryFigure(&XPS_POINT {
                x: start.x,
                y: start.y,
            })
        };

        let Ok(figure) = figure else {
            error!("couldn't create geometry figure: {:?}", figure.unwrap_err());
            return;
        };

        let segment_types = vec![XPS_SEGMENT_TYPE_LINE; points.len()];
        let mut segment_data: Vec<f32> = Vec::with_capacity(points.len() * 2);

        for point in points {
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

        unsafe { path.SetStrokeThickness(0.5).unwrap() };
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
}
