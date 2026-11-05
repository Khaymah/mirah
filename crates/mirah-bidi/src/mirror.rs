pub(crate) fn mirror(c: char) -> char {
    match c {
        '(' => ')',
        ')' => '(',
        '[' => ']',
        ']' => '[',
        '{' => '}',
        '}' => '{',
        '<' => '>',
        '>' => '<',
        '«' => '»',
        '»' => '«',
        _ => c,
    }
}

#[cfg(test)]
mod tests {
    use super::mirror;

    #[test]
    fn swaps_paired_brackets() {
        assert_eq!(mirror('('), ')');
        assert_eq!(mirror('«'), '»');
    }

    #[test]
    fn leaves_other_characters_untouched() {
        assert_eq!(mirror('a'), 'a');
        assert_eq!(mirror('ب'), 'ب');
    }
}
