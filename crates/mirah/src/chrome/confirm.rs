use super::style::{
    ACCENT, BODY_TOP, BOX_BG, DIM, Frame, Line, TEXT, blank, boxed, content_start, right_to_left,
    shaped, style,
};
use mirah_bidi::Shaping;
use std::ops::Range;

const GAP: usize = 3;

pub(crate) struct Dialog {
    pub(crate) frame: Frame,
    pub(crate) yes: Range<usize>,
    pub(crate) no: Range<usize>,
    pub(crate) row: usize,
}

impl Dialog {
    pub(crate) fn hit(&self, row: usize, column: usize) -> Option<bool> {
        if row != self.row {
            return None;
        }
        if self.yes.contains(&column) {
            Some(true)
        } else if self.no.contains(&column) {
            Some(false)
        } else {
            None
        }
    }
}

fn choice(text: &str, focused: bool, shaping: Shaping) -> Vec<mirah_bidi::Cell> {
    let s = if focused {
        style(BOX_BG, ACCENT, true)
    } else {
        style(TEXT, super::style::CHIP_BG, false)
    };
    shaped(&format!("  {text}  "), s, shaping)
}

pub(crate) fn draw(program: &str, yes_focused: bool, shaping: Shaping) -> Dialog {
    let yes = choice("إغلاق", yes_focused, shaping);
    let no = choice("إلغاء", !yes_focused, shaping);
    let width = yes.len() + no.len() + GAP;
    let mut cells = blank(width, style(TEXT, BOX_BG, false));
    right_to_left(&mut cells, width, &[yes.clone(), no.clone()], GAP);
    let title = shaped(
        &format!("إغلاق {program}؟"),
        style(TEXT, BOX_BG, true),
        shaping,
    );
    let hint = shaped(
        "Enter للتأكيد · Esc للإلغاء",
        style(DIM, BOX_BG, false),
        shaping,
    );
    let frame = boxed(&title, vec![Line::new(cells, false)], &hint, shaping);
    let at = content_start(frame.width, width);
    Dialog {
        yes: at + width - yes.len()..at + width,
        no: at..at + no.len(),
        row: BODY_TOP,
        frame,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_question_reads_right_to_left_with_yes_before_no() {
        let dialog = draw("claude", false, Shaping::Plain);
        let title: String = dialog.frame.rows[1]
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert!(title.contains("claude قالغإ"), "{title}");
        assert!(dialog.yes.start > dialog.no.start);
        assert_eq!(dialog.hit(BODY_TOP, dialog.yes.start), Some(true));
        assert_eq!(dialog.hit(BODY_TOP, dialog.no.start), Some(false));
        let row = &dialog.frame.rows[BODY_TOP];
        assert!(row[dialog.no.clone()].iter().all(|c| c.style.bg == ACCENT));
    }
}
