mod forms;
mod joining;
mod ligature;

use forms::{Forms, forms};
use joining::{is_transparent, joins_next, joins_prev, unshape};
use ligature::{LAM, lam_alef};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Shaped {
    Glyph(String),
    Absorbed,
}

fn marks(cell: &str) -> String {
    cell.chars().skip(1).collect()
}

fn previous_joiner(base: &[Option<char>], i: usize) -> Option<char> {
    base[..i]
        .iter()
        .rev()
        .copied()
        .find(|b| !b.is_some_and(is_transparent))
        .flatten()
}

fn next_joiner(base: &[Option<char>], i: usize) -> Option<char> {
    base[i + 1..]
        .iter()
        .copied()
        .find(|b| !b.is_some_and(is_transparent))
        .flatten()
}

fn contextual(f: &Forms, connects_prev: bool, connects_next: bool) -> char {
    match (connects_prev, connects_next) {
        (true, true) => f.medial.unwrap_or(f.final_),
        (true, false) => f.final_,
        (false, true) => f.initial.unwrap_or(f.isolated),
        (false, false) => f.isolated,
    }
}

pub fn shape<S: AsRef<str>>(cells: &[S]) -> Vec<Shaped> {
    let base: Vec<Option<char>> = cells
        .iter()
        .map(|s| s.as_ref().chars().next().map(unshape))
        .collect();
    let mut out = Vec::with_capacity(cells.len());
    let mut i = 0;
    while i < cells.len() {
        let next = next_joiner(&base, i);
        let connects_prev = joins_next(previous_joiner(&base, i));

        if base[i] == Some(LAM)
            && let Some((isolated, final_)) = next.and_then(lam_alef)
        {
            let glyph = if connects_prev { final_ } else { isolated };
            out.push(Shaped::Glyph(format!(
                "{glyph}{}{}",
                marks(cells[i].as_ref()),
                marks(cells[i + 1].as_ref())
            )));
            out.push(Shaped::Absorbed);
            i += 2;
            continue;
        }

        match base[i].and_then(forms) {
            Some(f) => {
                let connects_next = f.initial.is_some() && joins_prev(next);
                let glyph = contextual(&f, connects_prev, connects_next);
                out.push(Shaped::Glyph(format!(
                    "{glyph}{}",
                    marks(cells[i].as_ref())
                )));
            }
            None => out.push(Shaped::Glyph(cells[i].as_ref().to_string())),
        }
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
