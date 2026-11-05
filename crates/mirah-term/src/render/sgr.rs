use crate::sequences::CSI;
use mirah_bidi::{Color, Style};
use std::fmt::Write as _;

const BOLD: u8 = 1;
const DIM: u8 = 2;
const ITALIC: u8 = 3;
const UNDERLINE: u8 = 4;
const INVERSE: u8 = 7;

const STANDARD_COLORS: u8 = 8;
const BASIC_COLORS: u8 = 16;

struct Palette {
    standard: u8,
    bright: u8,
    extended: u8,
}

const FOREGROUND: Palette = Palette {
    standard: 30,
    bright: 90,
    extended: 38,
};

const BACKGROUND: Palette = Palette {
    standard: 40,
    bright: 100,
    extended: 48,
};

fn push_color(out: &mut String, color: Color, palette: &Palette) {
    let _ = match color {
        Color::Default => Ok(()),
        Color::Indexed(n) if n < STANDARD_COLORS => write!(out, ";{}", palette.standard + n),
        Color::Indexed(n) if n < BASIC_COLORS => {
            write!(out, ";{}", palette.bright + n - STANDARD_COLORS)
        }
        Color::Indexed(n) => write!(out, ";{};5;{n}", palette.extended),
        Color::Rgb(r, g, b) => write!(out, ";{};2;{r};{g};{b}", palette.extended),
    };
}

pub(crate) fn sgr(style: &Style) -> String {
    let mut out = format!("{CSI}0");
    for (on, code) in [
        (style.bold, BOLD),
        (style.dim, DIM),
        (style.italic, ITALIC),
        (style.underline, UNDERLINE),
        (style.inverse, INVERSE),
    ] {
        if on {
            let _ = write!(out, ";{code}");
        }
    }
    push_color(&mut out, style.fg, &FOREGROUND);
    push_color(&mut out, style.bg, &BACKGROUND);
    out.push('m');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_style_is_a_plain_reset() {
        assert_eq!(sgr(&Style::default()), "\x1b[0m");
    }

    #[test]
    fn encodes_attributes_and_every_color_kind() {
        let style = Style {
            fg: Color::Indexed(9),
            bg: Color::Rgb(1, 2, 3),
            bold: true,
            inverse: true,
            ..Style::default()
        };
        assert_eq!(sgr(&style), "\x1b[0;1;7;91;48;2;1;2;3m");
        let indexed = Style {
            fg: Color::Indexed(200),
            bg: Color::Indexed(2),
            ..Style::default()
        };
        assert_eq!(sgr(&indexed), "\x1b[0;38;5;200;42m");
    }
}
