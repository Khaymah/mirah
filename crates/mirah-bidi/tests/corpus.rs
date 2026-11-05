use mirah_bidi::{Cell, visual_row};
use std::path::PathBuf;

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/rows")
}

fn render(line: &str) -> String {
    let row: Vec<Cell> = line.chars().map(|c| Cell::plain(c.to_string())).collect();
    visual_row(&row)
        .cells
        .iter()
        .map(|c| c.text.as_str())
        .collect()
}

#[test]
fn every_row_in_the_corpus_matches_its_expected_visual_form() {
    let mut checked = 0;
    let mut failures = Vec::new();
    let mut files: Vec<PathBuf> = std::fs::read_dir(corpus_dir())
        .expect("corpus/rows exists")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .collect();
    files.sort();
    for file in files {
        let data = std::fs::read_to_string(&file).expect("corpus file is UTF-8");
        let mut lines = data.lines();
        while let (Some(logical), Some(visual)) = (lines.next(), lines.next()) {
            let logical = logical.strip_prefix("L|").expect("L| line");
            let expected = visual.strip_prefix("V|").expect("V| line");
            let got = render(logical);
            if got != expected {
                failures.push(format!(
                    "{}: {logical:?}\n  expected {expected:?}\n  got      {got:?}",
                    file.display()
                ));
            }
            checked += 1;
        }
    }
    assert!(checked > 0, "corpus is empty");
    assert!(
        failures.is_empty(),
        "{} of {checked} rows differ:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
