use super::classify::{base, is_rtl};
use mirah_bidi::Cell;

const CELL_PADDING: usize = 1;

fn mirrored(c: char) -> char {
    match c {
        '┌' => '┐',
        '┐' => '┌',
        '└' => '┘',
        '┘' => '└',
        '├' => '┤',
        '┤' => '├',
        '╭' => '╮',
        '╮' => '╭',
        '╰' => '╯',
        '╯' => '╰',
        _ => c,
    }
}

fn is_top(c: char) -> bool {
    matches!(c, '┌' | '╭')
}

fn is_border_joint(c: char) -> bool {
    matches!(
        c,
        '┌' | '┬'
            | '┐'
            | '├'
            | '┼'
            | '┤'
            | '└'
            | '┴'
            | '┘'
            | '╭'
            | '╮'
            | '╰'
            | '╯'
            | '│'
    )
}

struct Table {
    top: usize,
    bottom: usize,
    columns: Vec<usize>,
}

fn top_border(row: &[Cell]) -> Option<Vec<usize>> {
    let left = row.iter().position(|c| is_top(base(c)))?;
    let mut columns = vec![left];
    for (i, cell) in row.iter().enumerate().skip(left + 1) {
        match base(cell) {
            '─' => {}
            '┬' => columns.push(i),
            '┐' | '╮' => {
                columns.push(i);
                return (columns.len() >= 3).then_some(columns);
            }
            _ => return None,
        }
    }
    None
}

fn matches_columns(row: &[Cell], columns: &[usize]) -> bool {
    columns
        .iter()
        .all(|&c| row.get(c).is_some_and(|cell| is_border_joint(base(cell))))
        && !row.iter().any(|c| c.wide || c.continuation)
}

fn is_bottom(row: &[Cell], columns: &[usize]) -> bool {
    columns
        .first()
        .is_some_and(|&c| matches!(base(&row[c]), '└' | '╰'))
}

fn find_table(rows: &[Vec<Cell>], top: usize) -> Option<Table> {
    let columns = top_border(&rows[top])?;
    for (bottom, row) in rows.iter().enumerate().skip(top + 1) {
        if !matches_columns(row, &columns) {
            return None;
        }
        if is_bottom(row, &columns) {
            return Some(Table {
                top,
                bottom,
                columns,
            });
        }
    }
    None
}

fn has_rtl(rows: &[Vec<Cell>], table: &Table) -> bool {
    let (left, right) = (table.columns[0], table.columns[table.columns.len() - 1]);
    rows[table.top..=table.bottom]
        .iter()
        .any(|row| row[left..=right].iter().any(|c| c.text.chars().any(is_rtl)))
}

fn right_aligned(cells: &[Cell]) -> Vec<Cell> {
    let width = cells.len();
    let first = cells.iter().position(|c| base(c) != ' ');
    let last = cells.iter().rposition(|c| base(c) != ' ');
    let (Some(first), Some(last)) = (first, last) else {
        return cells.to_vec();
    };
    let content = &cells[first..=last];
    if content.len() + CELL_PADDING > width || content.iter().all(|c| base(c) == '─') {
        return cells.to_vec();
    }
    let pad = width - CELL_PADDING - content.len();
    let blank = Cell::plain(" ");
    std::iter::repeat_n(blank.clone(), pad)
        .chain(content.iter().cloned())
        .chain(std::iter::repeat_n(blank, CELL_PADDING))
        .collect()
}

fn mirror_row(row: &mut [Cell], columns: &[usize]) {
    let (left, right) = (columns[0], columns[columns.len() - 1]);
    let original = row[left..=right].to_vec();
    let at = |abs: usize| &original[abs - left];
    for &c in columns {
        let mut cell = at(c).clone();
        cell.text = mirrored(base(&cell)).to_string();
        row[left + right - c] = cell;
    }
    for pair in columns.windows(2) {
        let (start, end) = (pair[0] + 1, pair[1]);
        let content: Vec<Cell> = (start..end).map(|abs| at(abs).clone()).collect();
        let aligned = right_aligned(&content);
        let target = left + right - end + 1;
        for (offset, cell) in aligned.into_iter().enumerate() {
            row[target + offset] = cell;
        }
    }
}

pub(super) fn mirror_rtl_tables(mut rows: Vec<Vec<Cell>>) -> Vec<Vec<Cell>> {
    let mut i = 0;
    while i < rows.len() {
        match find_table(&rows, i) {
            Some(table) if has_rtl(&rows, &table) => {
                for row in &mut rows[table.top..=table.bottom] {
                    mirror_row(row, &table.columns);
                }
                i = table.bottom + 1;
            }
            Some(table) => i = table.bottom + 1,
            None => i += 1,
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(lines: &[&str]) -> Vec<Vec<Cell>> {
        lines
            .iter()
            .map(|l| l.chars().map(|c| Cell::plain(c.to_string())).collect())
            .collect()
    }

    fn texts(rows: &[Vec<Cell>]) -> Vec<String> {
        rows.iter()
            .map(|r| r.iter().map(|c| c.text.as_str()).collect())
            .collect()
    }

    #[test]
    fn mirrors_a_table_with_arabic() {
        let input = rows(&[
            "┌───────┬────┐",
            "│ الاسم │ id │",
            "├───────┼────┤",
            "│ سيد   │ 7  │",
            "└───────┴────┘",
        ]);
        assert_eq!(
            texts(&mirror_rtl_tables(input)),
            [
                "┌────┬───────┐",
                "│ id │ الاسم │",
                "├────┼───────┤",
                "│  7 │   سيد │",
                "└────┴───────┘",
            ]
        );
    }

    #[test]
    fn leaves_english_tables_alone() {
        let input = rows(&["┌───┬───┐", "│ a │ b │", "└───┴───┘"]);
        assert_eq!(texts(&mirror_rtl_tables(input.clone())), texts(&input));
    }

    #[test]
    fn leaves_a_broken_table_alone() {
        let input = rows(&["┌────┬───┐", "│ نص │ b", "└────┴───┘"]);
        assert_eq!(texts(&mirror_rtl_tables(input.clone())), texts(&input));
    }
}
