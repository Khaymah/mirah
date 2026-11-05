pub(crate) fn visual_column(logical_to_visual: &[usize], column: u16) -> u16 {
    logical_to_visual
        .get(usize::from(column))
        .and_then(|v| u16::try_from(*v).ok())
        .unwrap_or(column)
}

#[cfg(test)]
mod tests {
    use super::visual_column;

    #[test]
    fn maps_through_the_row_and_falls_back_past_its_end() {
        let map = [0, 3, 2, 1];
        assert_eq!(visual_column(&map, 1), 3);
        assert_eq!(visual_column(&map, 9), 9);
    }
}
