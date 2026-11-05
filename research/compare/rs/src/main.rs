use unicode_bidi::{BidiInfo, Level};

fn main() {
    let path = std::env::args().nth(1).unwrap_or_default();
    let data = std::fs::read_to_string(path).unwrap_or_default();
    let (mut total, mut level_ok, mut order_ok) = (0u32, 0u32, 0u32);
    let start = std::time::Instant::now();
    for line in data.lines().filter(|l| !l.starts_with('#') && !l.is_empty()) {
        let f: Vec<&str> = line.split(';').collect();
        let chars: Vec<char> = f[0]
            .split(' ')
            .filter_map(|h| u32::from_str_radix(h, 16).ok().and_then(char::from_u32))
            .collect();
        let text: String = chars.iter().collect();
        let para_level = match f[1] {
            "0" => Some(Level::ltr()),
            "1" => Some(Level::rtl()),
            _ => None,
        };
        let expected_levels: Vec<&str> = f[3].split(' ').collect();
        let expected_order: Vec<usize> = f[4].split(' ').filter_map(|s| s.parse().ok()).collect();
        total += 1;
        let info = BidiInfo::new(&text, para_level);
        let Some(para) = info.paragraphs.first() else { continue };
        let levels = info.reordered_levels_per_char(para, para.range.clone());
        let kept: Vec<usize> = (0..chars.len()).filter(|i| expected_levels[*i] != "x").collect();
        let levels_match = kept.iter().all(|i| levels.get(*i).map(|l| l.number().to_string()) == Some(expected_levels[*i].to_string()));
        if levels_match {
            level_ok += 1;
        }
        let kept_levels: Vec<Level> = kept.iter().map(|i| levels[*i]).collect();
        let order: Vec<usize> = BidiInfo::reorder_visual(&kept_levels).into_iter().map(|k| kept[k]).collect();
        if order == expected_order {
            order_ok += 1;
        }
    }
    println!("rust unicode-bidi 0.3.18: total={total} levels_ok={level_ok} ({:.2}%) order_ok={order_ok} ({:.2}%) time={:?}",
        level_ok as f64 * 100.0 / total as f64, order_ok as f64 * 100.0 / total as f64, start.elapsed());
}
