#[derive(Clone, Copy, PartialEq, Eq)]
enum Joining {
    Dual,
    Right,
    Causing,
}

struct Forms {
    isolated: char,
    final_: char,
    initial: Option<char>,
    medial: Option<char>,
}

fn joining(c: char) -> Option<Joining> {
    match c {
        '\u{0640}' => Some(Joining::Causing),
        _ => forms(c).map(|f| {
            if f.initial.is_some() {
                Joining::Dual
            } else {
                Joining::Right
            }
        }),
    }
}

fn dual(base: u32) -> Forms {
    Forms {
        isolated: char::from_u32(base).unwrap_or(' '),
        final_: char::from_u32(base + 1).unwrap_or(' '),
        initial: char::from_u32(base + 2),
        medial: char::from_u32(base + 3),
    }
}

fn right(base: u32) -> Forms {
    Forms {
        isolated: char::from_u32(base).unwrap_or(' '),
        final_: char::from_u32(base + 1).unwrap_or(' '),
        initial: None,
        medial: None,
    }
}

fn forms(c: char) -> Option<Forms> {
    let f = match c {
        '\u{0622}' => right(0xFE81),
        '\u{0623}' => right(0xFE83),
        '\u{0624}' => right(0xFE85),
        '\u{0625}' => right(0xFE87),
        '\u{0626}' => dual(0xFE89),
        '\u{0627}' => right(0xFE8D),
        '\u{0628}' => dual(0xFE8F),
        '\u{0629}' => right(0xFE93),
        '\u{062A}' => dual(0xFE95),
        '\u{062B}' => dual(0xFE99),
        '\u{062C}' => dual(0xFE9D),
        '\u{062D}' => dual(0xFEA1),
        '\u{062E}' => dual(0xFEA5),
        '\u{062F}' => right(0xFEA9),
        '\u{0630}' => right(0xFEAB),
        '\u{0631}' => right(0xFEAD),
        '\u{0632}' => right(0xFEAF),
        '\u{0633}' => dual(0xFEB1),
        '\u{0634}' => dual(0xFEB5),
        '\u{0635}' => dual(0xFEB9),
        '\u{0636}' => dual(0xFEBD),
        '\u{0637}' => dual(0xFEC1),
        '\u{0638}' => dual(0xFEC5),
        '\u{0639}' => dual(0xFEC9),
        '\u{063A}' => dual(0xFECD),
        '\u{0641}' => dual(0xFED1),
        '\u{0642}' => dual(0xFED5),
        '\u{0643}' => dual(0xFED9),
        '\u{0644}' => dual(0xFEDD),
        '\u{0645}' => dual(0xFEE1),
        '\u{0646}' => dual(0xFEE5),
        '\u{0647}' => dual(0xFEE9),
        '\u{0648}' => right(0xFEED),
        '\u{0649}' => right(0xFEEF),
        '\u{064A}' => dual(0xFEF1),
        '\u{067E}' => dual(0xFB56),
        '\u{0686}' => dual(0xFB7A),
        '\u{0698}' => right(0xFB8A),
        '\u{06A9}' => dual(0xFB8E),
        '\u{06AF}' => dual(0xFB92),
        '\u{06CC}' => dual(0xFBFC),
        _ => return None,
    };
    Some(f)
}

fn lam_alef(alef: char) -> Option<(char, char)> {
    match alef {
        '\u{0622}' => Some(('\u{FEF5}', '\u{FEF6}')),
        '\u{0623}' => Some(('\u{FEF7}', '\u{FEF8}')),
        '\u{0625}' => Some(('\u{FEF9}', '\u{FEFA}')),
        '\u{0627}' => Some(('\u{FEFB}', '\u{FEFC}')),
        _ => None,
    }
}

const LETTERS: [char; 41] = [
    '\u{0622}', '\u{0623}', '\u{0624}', '\u{0625}', '\u{0626}', '\u{0627}', '\u{0628}', '\u{0629}',
    '\u{062A}', '\u{062B}', '\u{062C}', '\u{062D}', '\u{062E}', '\u{062F}', '\u{0630}', '\u{0631}',
    '\u{0632}', '\u{0633}', '\u{0634}', '\u{0635}', '\u{0636}', '\u{0637}', '\u{0638}', '\u{0639}',
    '\u{063A}', '\u{0641}', '\u{0642}', '\u{0643}', '\u{0644}', '\u{0645}', '\u{0646}', '\u{0647}',
    '\u{0648}', '\u{0649}', '\u{064A}', '\u{067E}', '\u{0686}', '\u{0698}', '\u{06A9}', '\u{06AF}',
    '\u{06CC}',
];

fn unshape(c: char) -> char {
    if !('\u{FB50}'..='\u{FEFC}').contains(&c) {
        return c;
    }
    LETTERS
        .iter()
        .copied()
        .find(|&l| {
            forms(l).is_some_and(|f| {
                f.isolated == c || f.final_ == c || f.initial == Some(c) || f.medial == Some(c)
            })
        })
        .unwrap_or(c)
}

fn is_transparent(c: char) -> bool {
    unicode_bidi::bidi_class(c) == unicode_bidi::BidiClass::NSM
}

fn joins_next(c: Option<char>) -> bool {
    matches!(c.and_then(joining), Some(Joining::Dual | Joining::Causing))
}

fn joins_prev(c: Option<char>) -> bool {
    c.and_then(joining).is_some()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shaped {
    Glyph(String),
    Absorbed,
}

pub fn shape(cells: &[String]) -> Vec<Shaped> {
    let base: Vec<Option<char>> = cells
        .iter()
        .map(|s| s.chars().next().map(unshape))
        .collect();
    let mut out = Vec::with_capacity(cells.len());
    let mut i = 0;
    while i < cells.len() {
        let c = base[i];
        let marks: String = cells[i].chars().skip(1).collect();
        let prev = base[..i]
            .iter()
            .rev()
            .copied()
            .find(|b| !b.is_some_and(is_transparent))
            .flatten();
        let next = base[i + 1..]
            .iter()
            .copied()
            .find(|b| !b.is_some_and(is_transparent))
            .flatten();
        let connects_prev = joins_next(prev);

        if c == Some('\u{0644}')
            && let Some((iso, fin)) = next.and_then(lam_alef)
        {
            let glyph = if connects_prev { fin } else { iso };
            let alef_marks: String = cells[i + 1].chars().skip(1).collect();
            out.push(Shaped::Glyph(format!("{glyph}{marks}{alef_marks}")));
            out.push(Shaped::Absorbed);
            i += 2;
            continue;
        }

        let glyph = match c.and_then(forms) {
            Some(f) => {
                let connects_next = f.initial.is_some() && joins_prev(next);
                match (connects_prev, connects_next) {
                    (true, true) => f.medial.unwrap_or(f.final_),
                    (true, false) => f.final_,
                    (false, true) => f.initial.unwrap_or(f.isolated),
                    (false, false) => f.isolated,
                }
            }
            None => {
                out.push(Shaped::Glyph(cells[i].clone()));
                i += 1;
                continue;
            }
        };
        out.push(Shaped::Glyph(format!("{glyph}{marks}")));
        i += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cells(s: &str) -> Vec<String> {
        s.chars().map(String::from).collect()
    }

    fn glyphs(out: &[Shaped]) -> String {
        out.iter()
            .map(|g| match g {
                Shaped::Glyph(s) => s.clone(),
                Shaped::Absorbed => "_".into(),
            })
            .collect()
    }

    #[test]
    fn shapes_word_with_initial_medial_final() {
        assert_eq!(glyphs(&shape(&cells("بيت"))), "\u{FE91}\u{FEF4}\u{FE96}");
    }

    #[test]
    fn right_joining_letter_breaks_the_chain() {
        assert_eq!(glyphs(&shape(&cells("دار"))), "\u{FEA9}\u{FE8D}\u{FEAD}");
    }

    #[test]
    fn lam_alef_becomes_one_glyph_and_absorbs_alef() {
        assert_eq!(glyphs(&shape(&cells("لا"))), "\u{FEFB}_");
        assert_eq!(glyphs(&shape(&cells("سلام"))), "\u{FEB3}\u{FEFC}_\u{FEE1}");
    }

    #[test]
    fn keeps_harakat_attached_to_their_cell() {
        let input = vec!["ب\u{064E}".to_string(), "ت".to_string()];
        assert_eq!(glyphs(&shape(&input)), "\u{FE91}\u{064E}\u{FE96}");
    }

    #[test]
    fn harakat_in_their_own_cell_do_not_break_joining() {
        assert_eq!(glyphs(&shape(&cells("غّل"))), "\u{FECF}\u{0651}\u{FEDE}");
    }

    #[test]
    fn shapes_persian_letters() {
        assert_eq!(glyphs(&shape(&cells("پی"))), "\u{FB58}\u{FBFD}");
    }

    #[test]
    fn reshapes_text_that_an_app_already_shaped_wrongly() {
        let vim_output = cells("\u{FEE1}\u{FEAE}\u{FEA4}\u{FE91}\u{FE8D}");
        assert_eq!(
            glyphs(&shape(&vim_output)),
            "\u{FEE3}\u{FEAE}\u{FEA3}\u{FE92}\u{FE8E}"
        );
    }

    #[test]
    fn leaves_latin_untouched() {
        assert_eq!(glyphs(&shape(&cells("ab"))), "ab");
    }
}
