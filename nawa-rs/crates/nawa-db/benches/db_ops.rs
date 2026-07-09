//! NAWA-DB benchmarks via criterion.
//!
//! Run with: `cargo bench -p nawa-db`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nawa_db::{BloomFilter, DbEngine, SkipList, Value};

fn bench_put(c: &mut Criterion) {
    let mut group = c.benchmark_group("put");
    group.throughput(Throughput::Elements(1));

    for n in [100, 1_000, 10_000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(n), n, |b, &n| {
            b.iter_batched(
                || {
                    let db = DbEngine::open_in_memory();
                    (db, n)
                },
                |(db, n)| {
                    for i in 0..n {
                        db.put(format!("key:{i:06}"), Value::from_i64(i as i64))
                            .unwrap();
                    }
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }
    group.finish();
}

fn bench_get(c: &mut Criterion) {
    let db = DbEngine::open_in_memory();
    for i in 0..10_000u64 {
        db.put(format!("key:{i:06}"), Value::from_i64(i as i64)).unwrap();
    }

    let mut group = c.benchmark_group("get");
    group.throughput(Throughput::Elements(1));

    group.bench_function("existing_key", |b| {
        let mut i = 0u64;
        b.iter(|| {
            let key = format!("key:{i:06}");
            let v = db.get(&key).unwrap();
            assert_eq!(v.display(), i.to_string());
            i = (i + 1) % 10_000;
        });
    });

    group.bench_function("missing_key", |b| {
        b.iter(|| {
            assert!(db.get("missing:xxxxxxxxxx").is_none());
        });
    });

    group.finish();
}

fn bench_scan(c: &mut Criterion) {
    let db = DbEngine::open_in_memory();
    for i in 0..10_000u64 {
        db.put(format!("user:{i:06}"), Value::from_i64(i as i64)).unwrap();
    }

    let mut group = c.benchmark_group("scan");
    group.throughput(Throughput::Elements(1));

    group.bench_function("scan_100", |b| {
        b.iter(|| {
            let results = db.scan_prefix("user:", 100);
            assert_eq!(results.len(), 100);
        });
    });

    group.bench_function("scan_all", |b| {
        b.iter(|| {
            let results = db.scan_prefix("user:", 100_000);
            assert_eq!(results.len(), 10_000);
        });
    });

    group.finish();
}

fn bench_bloom(c: &mut Criterion) {
    let mut bf = BloomFilter::new(100_000, 0.01);
    for i in 0..100_000u32 {
        bf.insert(&i.to_le_bytes());
    }

    let mut group = c.benchmark_group("bloom");
    group.throughput(Throughput::Elements(1));

    group.bench_function("hit", |b| {
        let mut i = 0u32;
        b.iter(|| {
            assert!(bf.might_contain(&i.to_le_bytes()));
            i = (i + 1) % 100_000;
        });
    });

    group.bench_function("miss", |b| {
        let mut i = 100_000u32;
        b.iter(|| {
            bf.might_contain(&i.to_le_bytes());
            i = i.wrapping_add(1);
        });
    });

    group.finish();
}

fn bench_skip_list(c: &mut Criterion) {
    let mut group = c.benchmark_group("skip_list");
    group.throughput(Throughput::Elements(1));

    group.bench_function("insert", |b| {
        b.iter_batched(
            SkipList::<Vec<u8>, Vec<u8>>::new,
            |sl| {
                for i in 0..100u64 {
                    sl.insert(format!("key:{i:06}").into_bytes(), b"value".to_vec());
                }
            },
            criterion::BatchSize::SmallInput,
        );
    });

    let sl: SkipList<Vec<u8>, Vec<u8>> = SkipList::new();
    for i in 0..1000u64 {
        sl.insert(format!("key:{i:06}").into_bytes(), b"value".to_vec());
    }
    group.bench_function("get", |b| {
        let mut i = 0u64;
        b.iter(|| {
            let key = format!("key:{i:06}").into_bytes();
            sl.get(&key);
            i = (i + 1) % 1000;
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_put,
    bench_get,
    bench_scan,
    bench_bloom,
    bench_skip_list,
);
criterion_main!(benches);
