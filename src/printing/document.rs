use std::{
    error::Error,
    fmt::{Display, Formatter},
    path::Path,
};

use crate::config::Config;

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct DrawingAttributes {
    pub text_bold: bool,
    pub line_thickness: f32,
}

#[derive(Debug)]
pub enum DocumentBuildingError {
    NestedError(Box<dyn Error>),
    Error(String),
}

impl Display for DocumentBuildingError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentBuildingError::NestedError(e) => write!(f, "{}", e),
            DocumentBuildingError::Error(e) => write!(f, "{}", e),
        }
    }
}

impl Error for DocumentBuildingError {
    #[allow(deprecated)]
    fn description(&self) -> &str {
        match self {
            DocumentBuildingError::NestedError(e) => e.description(),
            DocumentBuildingError::Error(e) => e,
        }
    }

    fn cause(&self) -> Option<&dyn Error> {
        match self {
            DocumentBuildingError::NestedError(e) => Some(e.as_ref()),
            DocumentBuildingError::Error(_) => None,
        }
    }

    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            DocumentBuildingError::NestedError(e) => Some(e.as_ref()),
            DocumentBuildingError::Error(_) => None,
        }
    }
}

impl DrawingAttributes {
    pub const TEXT_BOLD: Self = Self {
        text_bold: true,
        line_thickness: 1.0,
    };

    pub const DEFAULT: Self = Self {
        text_bold: false,
        line_thickness: 1.0,
    };
}

pub trait DocumentBuilder {
    fn begin(&mut self) -> Result<(), DocumentBuildingError>;
    fn new_page(&mut self) -> Result<usize, DocumentBuildingError>;
    fn page_at(&mut self, index: usize) -> Option<&mut dyn PageBuilder>;
}

pub trait PageBuilder {
    fn get_dimnensions(&self) -> (f32, f32);

    fn add_outline_polygon(&mut self, points: &[Point], attributes: DrawingAttributes);
    fn add_text(
        &mut self,
        text: &str,
        x: f32,
        y: f32,
        font_size: f32,
        attributes: DrawingAttributes,
    );

    /// Adds a text block, that may include linebreaks (\r) by splitting the text into multiple lines.
    ///
    /// Returns the lowest y coordinate of the text. (useful for adding Elements below the text)
    /// the multiline text does not check for page overflow, so it is possible to write outside the page.
    /// This is due to the fact, that this method cannot create a pagebreak, as it holdes no reference to the containing document.
    fn add_multiline_text(
        &mut self,
        text: String,
        x: f32,
        y: f32,
        font_size: f32,
        attributes: DrawingAttributes,
    ) -> f32;

    /// Checks whether the given number of lines will overflow the page.
    ///
    /// NOTE: should be checked before calling [add_multiline_text()] to prevent overflow.
    fn will_multiline_overflow(
        &self,
        line_count: usize,
        y: f32,
        font_size: f32,
        attrs: DrawingAttributes,
    ) -> bool;

    /// Returns the number of lines, that can be added to the page before it overflows (given the configuration).
    /// see [will_multiline_overflow] for more information.
    fn max_lines_before_overflow(&self, y: f32, font_size: f32, attrs: DrawingAttributes) -> usize;

    fn add_horizontal_divider(&mut self, y: f32);

    fn add_img(&mut self, content: &[u8], x: f32, y: f32, width: usize, height: usize);
}

pub trait Printable {
    fn print(&self, times: usize, config: &Config);
}

pub trait Saveable {
    fn save(&self, path: &Path) -> Result<(), Box<dyn Error>>;
}

pub trait Document: DocumentBuilder + Printable + Saveable {}
