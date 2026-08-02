use std::{hint::black_box, time::Duration};

use collatz_engine::{PositiveInteger, PositiveU128, run, run_bigint, run_hybrid};
use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use rug::Integer;

fn positive(value: u128) -> PositiveU128 {
    PositiveU128::new(value).expect("benchmark literals are positive")
}

fn benchmark_engines(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("classical_engine_runs");
    let start_27 = positive(27);
    let exact_27 = PositiveInteger::from(start_27);

    group.bench_function("reference/27/limit=111", |bencher| {
        bencher.iter(|| black_box(run(black_box(start_27), black_box(111))))
    });

    group.bench_function("bigint/27/limit=111", |bencher| {
        bencher.iter_batched(
            || exact_27.clone(),
            |start| black_box(run_bigint(black_box(start), black_box(111))),
            BatchSize::SmallInput,
        );
    });

    group.bench_function("hybrid/27/limit=111", |bencher| {
        bencher.iter(|| black_box(run_hybrid(black_box(start_27), black_box(111))))
    });

    let maximum = positive(u128::MAX);
    group.bench_function("hybrid/u128_max/promotion/limit=1", |bencher| {
        bencher.iter(|| black_box(run_hybrid(black_box(maximum), black_box(1))))
    });

    let mersenne_256 = PositiveInteger::new((Integer::from(1) << 256) - 1)
        .expect("the generated Mersenne value is positive");
    group.bench_function("bigint/mersenne_256/limit=64", |bencher| {
        bencher.iter_batched(
            || mersenne_256.clone(),
            |start| black_box(run_bigint(black_box(start), black_box(64))),
            BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group! {
    name = engine_benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_millis(250))
        .measurement_time(Duration::from_millis(500));
    targets = benchmark_engines
}
criterion_main!(engine_benches);
