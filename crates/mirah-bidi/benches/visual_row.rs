use criterion::{Criterion, criterion_group, criterion_main};
use mirah_bidi::{Cell, visual_row};
use std::hint::black_box;

const WIDTH: usize = 80;

fn rows() -> Vec<Vec<Cell>> {
    let data = include_str!("../../../corpus/rows/agents.txt");
    data.lines()
        .filter_map(|l| l.strip_prefix("L|"))
        .map(|line| {
            let mut row: Vec<Cell> = line.chars().map(|c| Cell::plain(c.to_string())).collect();
            row.resize(WIDTH.max(row.len()), Cell::plain(" "));
            row
        })
        .collect()
}

fn bench(c: &mut Criterion) {
    let rows = rows();
    c.bench_function("visual_row/corpus", |b| {
        b.iter(|| {
            for row in &rows {
                black_box(visual_row(black_box(row)));
            }
        });
    });
}

criterion_group!(benches, bench);
criterion_main!(benches);
