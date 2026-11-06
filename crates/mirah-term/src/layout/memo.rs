use mirah_bidi::{BidiOptions, Cell, LogicalRow, VisualRow};
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;
const WORD: usize = 8;

#[derive(Default)]
struct Fx(u64);

impl Fx {
    fn add(&mut self, word: u64) {
        self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
    }
}

impl Hasher for Fx {
    fn write(&mut self, bytes: &[u8]) {
        for chunk in bytes.chunks(WORD) {
            let mut word = [0u8; WORD];
            word[..chunk.len()].copy_from_slice(chunk);
            self.add(u64::from_le_bytes(word));
        }
    }

    fn write_u8(&mut self, n: u8) {
        self.add(u64::from(n));
    }

    fn write_u64(&mut self, n: u64) {
        self.add(n);
    }

    fn write_usize(&mut self, n: usize) {
        self.add(n as u64);
    }

    fn finish(&self) -> u64 {
        self.0
    }
}

struct Entry<E, V> {
    row: Vec<Cell>,
    extra: E,
    value: V,
}

pub(crate) struct Memo<E, V> {
    current: HashMap<u64, Entry<E, V>>,
    previous: HashMap<u64, Entry<E, V>>,
}

impl<E, V> Default for Memo<E, V> {
    fn default() -> Self {
        Self {
            current: HashMap::new(),
            previous: HashMap::new(),
        }
    }
}

fn key<E: Hash>(row: &[Cell], extra: &E) -> u64 {
    let mut hasher = Fx::default();
    row.hash(&mut hasher);
    extra.hash(&mut hasher);
    hasher.finish()
}

impl<E: Hash + Eq + Clone, V: Clone> Memo<E, V> {
    pub(crate) fn get_or(&mut self, row: &[Cell], extra: &E, compute: impl FnOnce() -> V) -> V {
        let hash = key(row, extra);
        let same = |e: &Entry<E, V>| e.row == row && e.extra == *extra;
        if let Some(entry) = self.current.get(&hash).filter(|e| same(e)) {
            return entry.value.clone();
        }
        let entry = match self.previous.remove(&hash).filter(|e| same(e)) {
            Some(entry) => entry,
            None => Entry {
                row: row.to_vec(),
                extra: extra.clone(),
                value: compute(),
            },
        };
        let value = entry.value.clone();
        self.current.insert(hash, entry);
        value
    }

    fn end_frame(&mut self) {
        self.previous = std::mem::take(&mut self.current);
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.current.len() + self.previous.len()
    }
}

#[derive(Default)]
pub(crate) struct Caches {
    pub(crate) reorder: Memo<(Option<usize>, BidiOptions), VisualRow>,
    pub(crate) unreorder: Memo<Option<Vec<usize>>, Option<LogicalRow>>,
}

impl Caches {
    pub(crate) fn end_frame(&mut self) {
        self.reorder.end_frame();
        self.unreorder.end_frame();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(s: &str) -> Vec<Cell> {
        s.chars().map(|c| Cell::plain(c.to_string())).collect()
    }

    #[test]
    fn computes_once_while_the_row_stays_on_screen() {
        let mut memo: Memo<u8, u8> = Memo::default();
        let mut calls = 0;
        for _ in 0..3 {
            let value = memo.get_or(&row("ب"), &0, || {
                calls += 1;
                42
            });
            assert_eq!(value, 42);
            memo.end_frame();
        }
        assert_eq!(calls, 1);
    }

    #[test]
    fn a_different_row_or_extra_is_computed_again() {
        let mut memo: Memo<u8, u8> = Memo::default();
        assert_eq!(memo.get_or(&row("ب"), &0, || 1), 1);
        assert_eq!(memo.get_or(&row("ت"), &0, || 2), 2);
        assert_eq!(memo.get_or(&row("ب"), &1, || 3), 3);
        assert_eq!(memo.get_or(&row("ب"), &0, || 9), 1);
    }

    #[test]
    fn forgets_rows_that_left_the_screen_for_a_whole_frame() {
        let mut memo: Memo<u8, u8> = Memo::default();
        memo.get_or(&row("ب"), &0, || 1);
        memo.end_frame();
        memo.get_or(&row("ت"), &0, || 2);
        memo.end_frame();
        memo.end_frame();
        assert_eq!(memo.len(), 0);
    }
}
