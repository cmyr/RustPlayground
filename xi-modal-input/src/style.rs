use syntect::highlighting::{Color, FontStyle, Style as SyntectStyle};

/// Uniquely identifies a style, so only need to send styles to the frontend once.
pub(crate) type StyleId = u32;
pub(crate) type RgbaColor = u32;

#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, Serialize)]
pub(crate) struct Style {
    foreground: RgbaColor,
    background: RgbaColor,
    italic: bool,
    bold: bool,
    underline: bool,
}

impl From<SyntectStyle> for Style {
    fn from(src: SyntectStyle) -> Style {
        Style {
            foreground: rgba_from_syntect_color(src.foreground),
            background: rgba_from_syntect_color(src.background),
            italic: src.font_style.contains(FontStyle::ITALIC),
            bold: src.font_style.contains(FontStyle::BOLD),
            underline: src.font_style.contains(FontStyle::UNDERLINE),
        }
    }
}

fn rgba_from_syntect_color(color: Color) -> u32 {
    let Color { r, g, b, a } = color;
    ((a as u32) << 24) | ((r as u32) << 16) | ((g as u32) << 8) | (b as u32)
}
