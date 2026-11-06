use mirah_bidi::{BidiOptions, Cell, Shaping, Style, visual_row_with};
use std::path::PathBuf;

struct Case {
    logical: String,
    inverse: Vec<usize>,
    expected: String,
}

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus/rows")
}

fn parse(data: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    let mut current: Option<Case> = None;
    for line in data.lines() {
        if let Some(text) = line.strip_prefix("L|") {
            current = Some(Case {
                logical: text.to_string(),
                inverse: Vec::new(),
                expected: String::new(),
            });
        } else if let (Some(cols), Some(case)) = (line.strip_prefix("I|"), current.as_mut()) {
            case.inverse = cols
                .split(',')
                .filter_map(|c| c.trim().parse().ok())
                .collect();
        } else if let (Some(text), Some(mut case)) = (line.strip_prefix("V|"), current.take()) {
            case.expected = text.to_string();
            cases.push(case);
        }
    }
    cases
}

fn render(case: &Case, shaping: Shaping) -> String {
    let mut row: Vec<Cell> = case
        .logical
        .chars()
        .map(|c| Cell::plain(c.to_string()))
        .collect();
    if let Some(&last) = case.inverse.iter().max() {
        row.resize(row.len().max(last + 1), Cell::plain(" "));
    }
    for &i in &case.inverse {
        row[i].style = Style {
            inverse: true,
            ..Style::default()
        };
    }
    visual_row_with(
        &row,
        None,
        BidiOptions {
            shaping,
            rtl_base: false,
        },
    )
    .cells
    .iter()
    .map(|c| {
        if c.style.inverse {
            format!("[{}]", c.text)
        } else {
            c.text.to_string()
        }
    })
    .collect()
}

#[test]
fn every_row_in_the_corpus_matches_its_expected_visual_form() {
    let mut files: Vec<PathBuf> = std::fs::read_dir(corpus_dir())
        .expect("corpus/rows exists")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "txt"))
        .collect();
    files.sort();
    let mut checked = 0;
    let mut failures = Vec::new();
    for file in files {
        let data = std::fs::read_to_string(&file).expect("corpus file is UTF-8");
        let words = file
            .file_name()
            .is_some_and(|n| n.to_string_lossy().starts_with("plain"));
        let shaping = if words { Shaping::Plain } else { Shaping::Full };
        for case in parse(&data) {
            let got = render(&case, shaping);
            if got != case.expected {
                failures.push(format!(
                    "{}: {:?} inverse={:?}\n  expected {:?}\n  got      {:?}",
                    file.display(),
                    case.logical,
                    case.inverse,
                    case.expected,
                    got
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
