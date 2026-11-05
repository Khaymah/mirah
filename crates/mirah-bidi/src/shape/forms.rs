pub(crate) struct Forms {
    pub(crate) isolated: char,
    pub(crate) final_: char,
    pub(crate) initial: Option<char>,
    pub(crate) medial: Option<char>,
}

impl Forms {
    pub(crate) fn contains(&self, c: char) -> bool {
        self.isolated == c || self.final_ == c || self.initial == Some(c) || self.medial == Some(c)
    }
}

fn dual(base: u32) -> Forms {
    Forms {
        isolated: char::from_u32(base).unwrap_or(' '),
        final_: char::from_u32(base + 1).unwrap_or(' '),
        initial: char::from_u32(base + 2),
        medial: char::from_u32(base + 3),
    }
}

fn right(base: u32) -> Forms {
    Forms {
        isolated: char::from_u32(base).unwrap_or(' '),
        final_: char::from_u32(base + 1).unwrap_or(' '),
        initial: None,
        medial: None,
    }
}

pub(crate) const LETTERS: [char; 41] = [
    '\u{0622}', '\u{0623}', '\u{0624}', '\u{0625}', '\u{0626}', '\u{0627}', '\u{0628}', '\u{0629}',
    '\u{062A}', '\u{062B}', '\u{062C}', '\u{062D}', '\u{062E}', '\u{062F}', '\u{0630}', '\u{0631}',
    '\u{0632}', '\u{0633}', '\u{0634}', '\u{0635}', '\u{0636}', '\u{0637}', '\u{0638}', '\u{0639}',
    '\u{063A}', '\u{0641}', '\u{0642}', '\u{0643}', '\u{0644}', '\u{0645}', '\u{0646}', '\u{0647}',
    '\u{0648}', '\u{0649}', '\u{064A}', '\u{067E}', '\u{0686}', '\u{0698}', '\u{06A9}', '\u{06AF}',
    '\u{06CC}',
];

pub(crate) fn forms(c: char) -> Option<Forms> {
    let f = match c {
        '\u{0622}' => right(0xFE81),
        '\u{0623}' => right(0xFE83),
        '\u{0624}' => right(0xFE85),
        '\u{0625}' => right(0xFE87),
        '\u{0626}' => dual(0xFE89),
        '\u{0627}' => right(0xFE8D),
        '\u{0628}' => dual(0xFE8F),
        '\u{0629}' => right(0xFE93),
        '\u{062A}' => dual(0xFE95),
        '\u{062B}' => dual(0xFE99),
        '\u{062C}' => dual(0xFE9D),
        '\u{062D}' => dual(0xFEA1),
        '\u{062E}' => dual(0xFEA5),
        '\u{062F}' => right(0xFEA9),
        '\u{0630}' => right(0xFEAB),
        '\u{0631}' => right(0xFEAD),
        '\u{0632}' => right(0xFEAF),
        '\u{0633}' => dual(0xFEB1),
        '\u{0634}' => dual(0xFEB5),
        '\u{0635}' => dual(0xFEB9),
        '\u{0636}' => dual(0xFEBD),
        '\u{0637}' => dual(0xFEC1),
        '\u{0638}' => dual(0xFEC5),
        '\u{0639}' => dual(0xFEC9),
        '\u{063A}' => dual(0xFECD),
        '\u{0641}' => dual(0xFED1),
        '\u{0642}' => dual(0xFED5),
        '\u{0643}' => dual(0xFED9),
        '\u{0644}' => dual(0xFEDD),
        '\u{0645}' => dual(0xFEE1),
        '\u{0646}' => dual(0xFEE5),
        '\u{0647}' => dual(0xFEE9),
        '\u{0648}' => right(0xFEED),
        '\u{0649}' => right(0xFEEF),
        '\u{064A}' => dual(0xFEF1),
        '\u{067E}' => dual(0xFB56),
        '\u{0686}' => dual(0xFB7A),
        '\u{0698}' => right(0xFB8A),
        '\u{06A9}' => dual(0xFB8E),
        '\u{06AF}' => dual(0xFB92),
        '\u{06CC}' => dual(0xFBFC),
        _ => return None,
    };
    Some(f)
}
