fn main() {
    let path = std::env::args().nth(1).unwrap_or_default();
    let bytes = std::fs::read(path).unwrap_or_default();
    let rows = core::num::NonZeroU16::new(24).unwrap_or(core::num::NonZeroU16::MIN);
    let cols = core::num::NonZeroU16::new(80).unwrap_or(core::num::NonZeroU16::MIN);
    let mut p = vt100::Parser::new(rows, cols, 0);
    p.process(&bytes);
    let s = p.screen();
    for r in 0..24 {
        let line: String = (0..80u16)
            .filter_map(|c| s.cell(r, c))
            .filter(|c| !c.is_wide_continuation())
            .map(|c| if c.has_contents() { c.contents().to_string() } else { " ".into() })
            .collect();
        println!("{}", line.trim_end());
    }
    let (r, c) = s.cursor_position();
    println!("@cursor {r},{c}");
}
