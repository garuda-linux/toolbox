use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use toolbox_core::features::search::{Needle, folded};

criterion_group!(benches, bench_search);
criterion_main!(benches);

/// Driver-sized rows, like the Network page filters per keystroke.
fn rows() -> Vec<String> {
    (0_usize..500_usize)
        .map(|number| format!("iwlwifi-{number} Wireless-AC 9560 [Jefferson Peak]"))
        .collect()
}

fn bench_search(group: &mut Criterion) {
    let needle = Needle::new("wifi 95");
    let rows = rows();
    let folded: Vec<String> = rows.iter().map(|row| folded(row)).collect();

    group.bench_function("needle over unfolded rows", |bench| {
        bench.iter(|| {
            rows.iter()
                .filter(|row| needle.is_in(black_box(row)))
                .count()
        });
    });

    group.bench_function("needle over folded rows", |bench| {
        bench.iter(|| {
            folded
                .iter()
                .filter(|row| needle.is_in(black_box(row)))
                .count()
        });
    });
}
