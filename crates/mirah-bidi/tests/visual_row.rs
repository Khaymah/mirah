use mirah_bidi::{Cell, VisualRow, visual_row};

fn row(s: &str) -> Vec<Cell> {
    s.chars().map(|c| Cell::plain(c.to_string())).collect()
}

fn render(v: &VisualRow) -> String {
    v.cells.iter().map(|c| c.text.as_str()).collect()
}

#[test]
fn leaves_ltr_rows_untouched() {
    let r = row("hello │ world");
    assert_eq!(render(&visual_row(&r)), "hello │ world");
}

#[test]
fn keeps_frame_borders_in_place() {
    let out = render(&visual_row(&row("│ مرحبا بك  │  status: جاهز")));
    assert!(out.starts_with("│ "));
    assert_eq!(out.chars().filter(|c| *c == '│').count(), 2);
    assert_eq!(out.chars().nth(12), Some('│'));
    assert!(out.contains("status: "));
}

#[test]
fn reverses_arabic_and_keeps_latin_order() {
    let out = render(&visual_row(&row("افتح src/main.rs الآن")));
    assert!(out.contains("src/main.rs"));
    let latin = out.find("src").unwrap_or(0);
    let open = out.find('\u{FEA2}').unwrap_or(0);
    assert!(open > latin);
}

#[test]
fn keeps_prompt_prefix_on_the_left() {
    let out = render(&visual_row(&row("> مرحبا")));
    assert!(out.starts_with("> "));
}

#[test]
fn keeps_row_width_after_lam_alef() {
    let r = row("│ السلام عليكم │");
    let v = visual_row(&r);
    assert_eq!(v.cells.len(), r.len());
    assert_eq!(v.cells.last().map(|c| c.text.as_str()), Some("│"));
}

#[test]
fn maps_every_logical_column_to_a_unique_visual_column() {
    let r = row("> اكتب هنا 42 مرة");
    let v = visual_row(&r);
    let mut seen = v.logical_to_visual.clone();
    seen.sort_unstable();
    seen.dedup();
    assert_eq!(seen.len(), r.len());
}

#[test]
fn mirrors_brackets_inside_rtl_runs() {
    let out = render(&visual_row(&row("مرحبا (عالم)")));
    assert!(out.contains('(') && out.contains(')'));
    assert!(out.find('(').unwrap_or(0) < out.find(')').unwrap_or(0));
}

#[test]
fn a_wide_emoji_moves_with_its_sentence() {
    let mut r = row("⚠ ب");
    r[0].wide = true;
    r.insert(
        1,
        Cell {
            continuation: true,
            ..Cell::plain("")
        },
    );
    let v = mirah_bidi::visual_row_with(
        &r,
        None,
        mirah_bidi::BidiOptions {
            shaping: mirah_bidi::Shaping::Plain,
            rtl_base: true,
        },
    );
    assert_eq!(render(&v), "ب ⚠");
    assert!(v.cells[2].wide);
    assert!(v.cells[3].continuation);
    assert_eq!(v.logical_to_visual, [2, 3, 1, 0]);
}
