use unicode_bidi::{BidiInfo, Level};

const TEST_FILE_ENV: &str = "MIRAH_BIDI_CHARACTER_TEST";

struct Case {
    text: String,
    paragraph: Option<Level>,
    levels: Vec<Option<u8>>,
    order: Vec<usize>,
}

fn parse(line: &str) -> Option<Case> {
    let fields: Vec<&str> = line.split(';').collect();
    let [code_points, direction, _, levels, order] = fields.as_slice() else {
        return None;
    };
    let text = code_points
        .split(' ')
        .filter_map(|h| u32::from_str_radix(h, 16).ok().and_then(char::from_u32))
        .collect();
    let paragraph = match *direction {
        "0" => Some(Level::ltr()),
        "1" => Some(Level::rtl()),
        _ => None,
    };
    let levels = levels.split(' ').map(|l| l.parse().ok()).collect();
    let order = order.split(' ').filter_map(|i| i.parse().ok()).collect();
    Some(Case {
        text,
        paragraph,
        levels,
        order,
    })
}

fn passes(case: &Case) -> bool {
    let info = BidiInfo::new(&case.text, case.paragraph);
    let Some(para) = info.paragraphs.first() else {
        return false;
    };
    let levels = info.reordered_levels_per_char(para, para.range.clone());
    let kept: Vec<usize> = (0..case.levels.len())
        .filter(|i| case.levels[*i].is_some())
        .collect();
    let levels_match = kept
        .iter()
        .all(|i| levels.get(*i).map(|l| l.number()) == case.levels[*i]);
    let kept_levels: Vec<Level> = kept.iter().map(|i| levels[*i]).collect();
    let order: Vec<usize> = BidiInfo::reorder_visual(&kept_levels)
        .into_iter()
        .map(|k| kept[k])
        .collect();
    levels_match && order == case.order
}

#[test]
#[ignore = "needs BidiCharacterTest.txt; set MIRAH_BIDI_CHARACTER_TEST"]
fn unicode_bidi_passes_the_official_character_test() {
    let path = std::env::var(TEST_FILE_ENV).expect("MIRAH_BIDI_CHARACTER_TEST is set");
    let data = std::fs::read_to_string(path).expect("test file is readable");
    let cases: Vec<Case> = data
        .lines()
        .filter(|l| !l.starts_with('#') && !l.is_empty())
        .filter_map(parse)
        .collect();
    let failed = cases.iter().filter(|c| !passes(c)).count();
    assert!(!cases.is_empty());
    assert_eq!(failed, 0, "{failed} of {} cases failed", cases.len());
}
