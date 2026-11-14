use super::confirm::Dialog;
use super::style::{
    ACCENT, BODY_TOP, BOX_BG, CHIP_BG, DIM, Line, TEXT, UPDATE, blank, boxed, content_start,
    right_to_left, shaped, style,
};
use mirah_bidi::{Cell, Shaping};

const GAP: usize = 3;
const SPINNER_DOTS: usize = 3;
const STEP_GAP: usize = 1;
const BUTTONS_ROW: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Step {
    Download,
    Verify,
    Install,
}

const STEPS: [(Step, &str); 3] = [
    (Step::Download, "التنزيل"),
    (Step::Verify, "التحقق"),
    (Step::Install, "التثبيت"),
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Updating {
    Offer {
        current: String,
        version: String,
        now: bool,
    },
    Working {
        version: String,
        step: Step,
        tick: usize,
    },
    Installed {
        version: String,
        restart: bool,
    },
    Failed,
}

fn button(text: &str, focused: bool, shaping: Shaping) -> Vec<Cell> {
    let s = if focused {
        style(BOX_BG, ACCENT, true)
    } else {
        style(TEXT, CHIP_BG, false)
    };
    shaped(&format!("  {text}  "), s, shaping)
}

fn line(parts: &[Vec<Cell>], gap: usize) -> Vec<Cell> {
    let width = parts.iter().map(Vec::len).sum::<usize>() + gap * parts.len().saturating_sub(1);
    let mut cells = blank(width, style(TEXT, BOX_BG, false));
    right_to_left(&mut cells, width, parts, gap);
    cells
}

fn spinner(tick: usize, shaping: Shaping) -> Vec<Cell> {
    let lit = tick % SPINNER_DOTS;
    let dots: Vec<Vec<Cell>> = (0..SPINNER_DOTS)
        .map(|i| {
            let (glyph, fg) = if i == lit {
                ("●", ACCENT)
            } else {
                ("○", DIM)
            };
            shaped(glyph, style(fg, BOX_BG, false), shaping)
        })
        .collect();
    line(&dots, 1)
}

fn step_text(step: Step, version: &str) -> String {
    match step {
        Step::Download => format!("جارٍ تنزيل الإصدار {version}…"),
        Step::Verify => "جارٍ التحقق من الملف…".to_string(),
        Step::Install => "جارٍ التثبيت…".to_string(),
    }
}

fn steps(current: Step, shaping: Shaping) -> Vec<Cell> {
    let mut parts = Vec::new();
    for (i, (step, name)) in STEPS.iter().enumerate() {
        if i > 0 {
            parts.push(shaped("←", style(DIM, BOX_BG, false), shaping));
        }
        let fg = match step.cmp(&current) {
            std::cmp::Ordering::Less => TEXT,
            std::cmp::Ordering::Equal => ACCENT,
            std::cmp::Ordering::Greater => DIM,
        };
        parts.push(shaped(name, style(fg, BOX_BG, *step == current), shaping));
    }
    line(&parts, STEP_GAP)
}

fn title(text: &str, shaping: Shaping) -> Vec<Cell> {
    let mut brand = shaped("◆ ", style(ACCENT, BOX_BG, true), shaping);
    brand.extend(shaped(text, style(TEXT, BOX_BG, true), shaping));
    line(&[brand], 0)
}

fn hint(text: &str, shaping: Shaping) -> Vec<Cell> {
    shaped(text, style(DIM, BOX_BG, false), shaping)
}

fn no_buttons(frame: super::style::Frame) -> Dialog {
    Dialog {
        frame,
        yes: 0..0,
        no: 0..0,
        row: BODY_TOP,
    }
}

pub(crate) fn draw(state: &Updating, shaping: Shaping) -> Dialog {
    match state {
        Updating::Offer {
            current,
            version,
            now,
        } => {
            let yes = button("تحديث الآن", *now, shaping);
            let no = button("لاحقًا", !*now, shaping);
            let width = yes.len() + no.len() + GAP;
            let mut buttons = blank(width, style(TEXT, BOX_BG, false));
            right_to_left(&mut buttons, width, &[yes.clone(), no.clone()], GAP);
            let detail = shaped(
                &format!("الإصدار {version} متاح، ولديك {current}"),
                style(TEXT, BOX_BG, false),
                shaping,
            );
            let frame = boxed(
                &title("نسخة جديدة من مرآة", shaping),
                vec![
                    Line::new(detail, false),
                    Line::new(Vec::new(), false),
                    Line::new(buttons, false),
                ],
                &hint("← → للاختيار · Enter للتأكيد · Esc لاحقًا", shaping),
                shaping,
            );
            let at = content_start(frame.width, width);
            Dialog {
                yes: at + width - yes.len()..at + width,
                no: at..at + no.len(),
                row: BODY_TOP + BUTTONS_ROW,
                frame,
            }
        }
        Updating::Working {
            version,
            step,
            tick,
        } => {
            let status = line(
                &[
                    spinner(*tick, shaping),
                    shaped(
                        &step_text(*step, version),
                        style(TEXT, BOX_BG, false),
                        shaping,
                    ),
                ],
                2,
            );
            no_buttons(boxed(
                &title("تحديث مرآة", shaping),
                vec![
                    Line::new(status, false),
                    Line::new(Vec::new(), false),
                    Line::new(steps(*step, shaping), false),
                ],
                &hint("Esc للإخفاء، ويستمر التحديث", shaping),
                shaping,
            ))
        }
        Updating::Installed { version, restart } => {
            let done = line(
                &[
                    shaped("●", style(UPDATE, BOX_BG, false), shaping),
                    shaped(
                        &format!("ثُبّت الإصدار {version}"),
                        style(TEXT, BOX_BG, true),
                        shaping,
                    ),
                ],
                2,
            );
            let next = if *restart {
                "جارٍ إعادة تشغيل مرآة…"
            } else {
                "يعمل في المرة القادمة التي تفتح فيها مرآة"
            };
            no_buttons(boxed(
                &title("تحديث مرآة", shaping),
                vec![
                    Line::new(done, false),
                    Line::new(shaped(next, style(DIM, BOX_BG, false), shaping), false),
                ],
                &hint("Enter للإغلاق", shaping),
                shaping,
            ))
        }
        Updating::Failed => no_buttons(boxed(
            &title("تحديث مرآة", shaping),
            vec![Line::new(
                shaped(
                    "تعذّر التحديث، والنسخة الحالية كما هي",
                    style(TEXT, BOX_BG, false),
                    shaping,
                ),
                false,
            )],
            &hint("التفاصيل في ملف السجل · Enter للإغلاق", shaping),
            shaping,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(cells: &[Cell]) -> String {
        cells.iter().map(|c| c.text.as_str()).collect()
    }

    fn all() -> Vec<Updating> {
        vec![
            Updating::Offer {
                current: "0.4.1".into(),
                version: "0.4.2".into(),
                now: true,
            },
            Updating::Working {
                version: "0.4.2".into(),
                step: Step::Verify,
                tick: 4,
            },
            Updating::Installed {
                version: "0.4.2".into(),
                restart: true,
            },
            Updating::Failed,
        ]
    }

    #[test]
    fn every_update_box_fits_sixty_columns_and_uses_safe_glyphs() {
        for state in all() {
            let dialog = draw(&state, Shaping::Plain);
            assert!(
                dialog.frame.width <= 60,
                "{state:?} is {} wide",
                dialog.frame.width
            );
            for row in &dialog.frame.rows {
                for cell in row {
                    for c in cell.text.chars() {
                        assert!(
                            c.is_ascii()
                                || ('\u{0600}'..='\u{06FF}').contains(&c)
                                || "≡●■✕+↻↺◆◉○•‹›▌▐┃│─╭╮╰╯├┤↑↓←→·…".contains(c),
                            "{c:?} in {state:?}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn update_now_sits_right_of_later_and_both_can_be_clicked() {
        let dialog = draw(&all()[0], Shaping::Plain);
        assert!(dialog.yes.start > dialog.no.start);
        let row = BODY_TOP + BUTTONS_ROW;
        assert_eq!(dialog.hit(row, dialog.yes.start), Some(true));
        assert_eq!(dialog.hit(row, dialog.no.start), Some(false));
        assert_eq!(dialog.hit(BODY_TOP, dialog.yes.start), None);
    }

    #[test]
    fn progress_names_the_current_step_and_moves_the_spinner() {
        let a = draw(
            &Updating::Working {
                version: "0.4.2".into(),
                step: Step::Download,
                tick: 0,
            },
            Shaping::Plain,
        );
        let b = draw(
            &Updating::Working {
                version: "0.4.2".into(),
                step: Step::Download,
                tick: 1,
            },
            Shaping::Plain,
        );
        assert_ne!(text(&a.frame.rows[BODY_TOP]), text(&b.frame.rows[BODY_TOP]));
        assert!(text(&a.frame.rows[BODY_TOP]).contains("0.4.2"));
        assert_eq!(a.hit(BODY_TOP, 10), None);
    }
}
