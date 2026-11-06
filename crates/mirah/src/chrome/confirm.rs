use super::bar::{TEXT, shaped, style};
use mirah_bidi::{Cell, Shaping};
use std::ops::Range;

pub(crate) struct Dialog {
    pub(crate) cells: Vec<Cell>,
    pub(crate) yes: Range<usize>,
    pub(crate) no: Range<usize>,
}

pub(crate) fn draw(program: &str, yes_focused: bool, shaping: Shaping) -> Dialog {
    let plain = style(TEXT, false, false);
    let no = shaped(" لا ", style(TEXT, true, !yes_focused), shaping);
    let yes = shaped(" نعم ", style(TEXT, true, yes_focused), shaping);
    let question = shaped(&format!(" تقفل {program}؟ "), plain, shaping);
    let gap = shaped("  ", plain, shaping);
    let mut cells = Vec::new();
    let no_range = cells.len()..cells.len() + no.len();
    cells.extend(no);
    cells.extend(gap.iter().cloned());
    let yes_range = cells.len()..cells.len() + yes.len();
    cells.extend(yes);
    cells.extend(gap);
    cells.extend(question);
    Dialog {
        cells,
        yes: yes_range,
        no: no_range,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_question_reads_right_to_left_with_yes_before_no() {
        let dialog = draw("claude", false, Shaping::Plain);
        let text: String = dialog.cells.iter().map(|c| c.text.as_str()).collect();
        assert!(text.trim_end().ends_with("claude لفقت"), "{text}");
        assert!(dialog.yes.start > dialog.no.start);
        assert!(
            dialog.cells[dialog.no.clone()]
                .iter()
                .all(|c| c.style.inverse)
        );
    }
}
