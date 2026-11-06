use super::classify::{base, blank_like, is_blank, is_rtl};
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

fn is_left_edge(c: char) -> bool {
    matches!(c, '┌' | '├' | '└' | '╭' | '╰')
}

fn is_right_edge(c: char) -> bool {
    matches!(c, '┐' | '┤' | '┘' | '╮' | '╯')
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

fn border_columns(row: &[Cell]) -> Option<Vec<usize>> {
    let left = row.iter().position(|c| is_left_edge(base(c)))?;
    let mut columns = vec![left];
    for (i, cell) in row.iter().enumerate().skip(left + 1) {
        match base(cell) {
            '─' => {}
            '┬' | '┼' | '┴' => columns.push(i),
            c if is_right_edge(c) => {
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
        && columns.iter().all(|&c| {
            row.get(c)
                .is_some_and(|cell| !cell.wide && !cell.continuation)
        })
}

fn edge_is(row: &[Cell], columns: &[usize], edges: [char; 2]) -> bool {
    columns
        .first()
        .and_then(|&c| row.get(c))
        .is_some_and(|cell| edges.contains(&base(cell)))
}

fn table_at(rows: &[Vec<Cell>], seed: usize) -> Option<Table> {
    let columns = border_columns(&rows[seed])?;
    let mut top = seed;
    while !edge_is(&rows[top], &columns, ['┌', '╭'])
        && top > 0
        && matches_columns(&rows[top - 1], &columns)
    {
        top -= 1;
    }
    let mut bottom = seed;
    while !edge_is(&rows[bottom], &columns, ['└', '╰'])
        && bottom + 1 < rows.len()
        && matches_columns(&rows[bottom + 1], &columns)
    {
        bottom += 1;
    }
    (bottom > top).then_some(Table {
        top,
        bottom,
        columns,
    })
}

fn find_tables(rows: &[Vec<Cell>]) -> Vec<Table> {
    let mut tables = Vec::new();
    let mut i = 0;
    while i < rows.len() {
        match table_at(rows, i) {
            Some(table) => {
                i = table.bottom + 1;
                tables.push(table);
            }
            None => i += 1,
        }
    }
    tables
}

pub(super) fn frame_hints(rows: &[Vec<Cell>]) -> Vec<Option<Vec<usize>>> {
    let mut hints = vec![None; rows.len()];
    for table in find_tables(rows) {
        for hint in &mut hints[table.top..=table.bottom] {
            *hint = Some(table.columns.clone());
        }
    }
    hints
}

fn has_rtl(rows: &[Vec<Cell>], table: &Table) -> bool {
    let (left, right) = (table.columns[0], table.columns[table.columns.len() - 1]);
    rows[table.top..=table.bottom]
        .iter()
        .any(|row| row[left..=right].iter().any(|c| c.text.chars().any(is_rtl)))
}

fn right_aligned(cells: &[Cell]) -> Vec<Cell> {
    let width = cells.len();
    let first = cells.iter().position(|c| !is_blank(c));
    let last = cells.iter().rposition(|c| !is_blank(c));
    let (Some(first), Some(last)) = (first, last) else {
        return cells.to_vec();
    };
    let content = &cells[first..=last];
    if content.len() + CELL_PADDING > width || content.iter().all(|c| base(c) == '─') {
        return cells.to_vec();
    }
    let pad = width - CELL_PADDING - content.len();
    let blank = blank_like(cells.first());
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
        cell.text = std::iter::once(mirrored(base(&cell))).collect();
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

fn move_right(rows: &mut [Vec<Cell>], table: &Table) {
    let (left, right) = (table.columns[0], table.columns[table.columns.len() - 1]);
    let width = rows[table.top..=table.bottom]
        .iter()
        .map(Vec::len)
        .min()
        .unwrap_or(0);
    let Some(target) = width.checked_sub(1 + left) else {
        return;
    };
    if target <= right {
        return;
    }
    let shift = target - right;
    let free = rows[table.top..=table.bottom]
        .iter()
        .all(|row| row[right + 1..=target].iter().all(is_blank));
    if !free {
        return;
    }
    for row in &mut rows[table.top..=table.bottom] {
        row[left..=target].rotate_right(shift);
    }
}

pub(super) fn mirror_rtl_tables(rows: Vec<Vec<Cell>>, settled: &[bool]) -> Vec<Vec<Cell>> {
    let mut rows = rows;
    for table in find_tables(&rows) {
        if !has_rtl(&rows, &table) {
            continue;
        }
        let unsettled = !(table.top..=table.bottom).any(|r| settled.get(r) == Some(&true));
        if unsettled {
            for row in &mut rows[table.top..=table.bottom] {
                mirror_row(row, &table.columns);
            }
        }
        move_right(&mut rows, &table);
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
    fn a_narrow_arabic_table_moves_to_the_right_edge() {
        let input = rows(&[
            "  ┌───────┬────┐      ",
            "  │ الاسم │ id │      ",
            "  └───────┴────┘      ",
            "  after               ",
        ]);
        assert_eq!(
            texts(&mirror_rtl_tables(input, &[])),
            [
                "      ┌────┬───────┐  ",
                "      │ id │ الاسم │  ",
                "      └────┴───────┘  ",
                "  after               ",
            ]
        );
        let busy = rows(&["┌───────┬────┐ x", "│ الاسم │ id │  ", "└───────┴────┘  "]);
        assert_eq!(texts(&mirror_rtl_tables(busy, &[]))[0], "┌────┬───────┐ x");
    }

    #[test]
    fn a_table_the_program_already_mirrored_still_moves_right() {
        let input = rows(&[
            "┌────┬───────┐    ",
            "│ id │ الاسم │    ",
            "└────┴───────┘    ",
        ]);
        let out = mirror_rtl_tables(input, &[true, true, true]);
        assert_eq!(texts(&out)[1], "    │ id │ الاسم │");
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
            texts(&mirror_rtl_tables(input, &[])),
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
        assert_eq!(texts(&mirror_rtl_tables(input.clone(), &[])), texts(&input));
    }

    #[test]
    fn mirrors_a_table_cut_off_by_scrolling() {
        let input = rows(&["│ سيد   │ 7  │", "├───────┼────┤", "│ علي   │ 9  │"]);
        assert_eq!(
            texts(&mirror_rtl_tables(input, &[])),
            ["│  7 │   سيد │", "├────┼───────┤", "│  9 │   علي │"]
        );
    }

    #[test]
    fn mirrors_rows_with_wide_emoji_and_keeps_each_emoji_whole() {
        let mut input = rows(&[
            "┌──────┬─────────┐",
            "│ سيد  │ Rust X  │",
            "└──────┴─────────┘",
        ]);
        input[1][14].wide = true;
        input[1][15] = Cell {
            continuation: true,
            ..Cell::plain("")
        };
        let out = mirror_rtl_tables(input, &[]);
        assert_eq!(texts(&out)[0], "┌─────────┬──────┐");
        let row = &out[1];
        let wide = row.iter().position(|c| c.wide).expect("emoji kept");
        assert!(row[wide + 1].continuation, "{:?}", texts(&out)[1]);
    }

    #[test]
    fn leaves_a_broken_table_alone() {
        let input = rows(&["┌────┬───┐", "│ نص │ b", "└────┴───┘"]);
        assert_eq!(texts(&mirror_rtl_tables(input.clone(), &[])), texts(&input));
    }
}

#[cfg(test)]
mod hint_tests {
    use super::*;

    #[test]
    fn rows_under_a_top_border_get_its_columns() {
        let input: Vec<Vec<Cell>> = ["x", "┌──┬──┐", "│ ب│ a│", "└──┴──┘", "y"]
            .iter()
            .map(|l| l.chars().map(|c| Cell::plain(c.to_string())).collect())
            .collect();
        let hints = frame_hints(&input);
        assert_eq!(hints[0], None);
        assert_eq!(hints[2], Some(vec![0, 3, 6]));
        assert_eq!(hints[3], Some(vec![0, 3, 6]));
        assert_eq!(hints[4], None);
    }
}

#[cfg(test)]
mod settled_tests {
    use super::*;

    #[test]
    fn leaves_a_table_with_a_settled_row_alone() {
        let input: Vec<Vec<Cell>> = ["┌──┬──┐", "│ ب│ a│", "└──┴──┘"]
            .iter()
            .map(|l| l.chars().map(|c| Cell::plain(c.to_string())).collect())
            .collect();
        assert_eq!(
            mirror_rtl_tables(input.clone(), &[false, true, false]),
            input
        );
    }
}
