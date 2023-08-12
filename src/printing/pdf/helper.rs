#[macro_export]
macro_rules! points_to_mm {
    ($x:expr) => {
        $x * 0.35278
    };
}

#[macro_export]
macro_rules! line_thickness {
    ($attrs:ident) => {
        unsafe { $attrs.size.line_thickness }
    };
}

#[macro_export]
macro_rules! font_size {
    ($attrs:ident) => {
        unsafe { $attrs.size.font_size }
    };
}

#[macro_export]
macro_rules! text_line_height {
    ($attrs:ident) => {
        unsafe { $attrs.size.font_size + 1.0 }
    };
    ($attrs:expr) => {
        unsafe { $attrs.size.font_size + 1.0 }
    };
}
