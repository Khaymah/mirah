use mirah_bidi as segment;

fn main() {
    let data = std::fs::read_to_string(std::env::args().nth(1).unwrap_or_default()).unwrap_or_default();
    let rows: Vec<Vec<segment::Cell>> = data
        .lines()
        .map(|l| {
            let mut v: Vec<segment::Cell> = l.chars().map(|c| segment::Cell::plain(c.to_string())).collect();
            v.resize(80, segment::Cell::plain(" "));
            v
        })
        .collect();
    for r in &rows {
        let v = segment::visual_row(r);
        println!("{}", v.cells.iter().map(|c| c.text.as_str()).collect::<String>().trim_end());
    }
    let start = std::time::Instant::now();
    let mut n = 0usize;
    for _ in 0..5000 {
        for r in &rows {
            n += segment::visual_row(r).cells.len();
        }
    }
    eprintln!("rust: {} rows in {:?} ({:.2} µs/row) {n}", 5000 * rows.len(), start.elapsed(), start.elapsed().as_secs_f64() * 1e6 / (5000 * rows.len()) as f64);
}
