//! NAWA-uring benchmarks.
//!
//! Compare io_uring pipeline throughput vs tokio fallback.
//! Run with: `cargo bench -p nawa-uring`

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use nawa_uring::{NawaUring, OpCode, PipelineConfig, SubmissionEntry};

fn bench_submit_single(c: &mut Criterion) {
    let mut group = c.benchmark_group("submit_single");
    group.throughput(Throughput::Elements(1));

    group.bench_function("default_config", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            let uring = NawaUring::default().unwrap();
            let entry = SubmissionEntry::new(OpCode::Fsync, 1, 0);
            uring.submit(entry).await.unwrap();
        });
    });

    group.bench_function("high_throughput_config", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            let uring = NawaUring::high_throughput().unwrap();
            let entry = SubmissionEntry::new(OpCode::Fsync, 1, 0);
            uring.submit(entry).await.unwrap();
        });
    });

    group.finish();
}

fn bench_submit_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("submit_batch");
    group.throughput(Throughput::Elements(1));

    for batch_size in [10, 100, 1000].iter() {
        group.bench_with_input(
            BenchmarkId::new("batch", batch_size),
            batch_size,
            |b, &size| {
                b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
                    let uring = NawaUring::default().unwrap();
                    let entries: Vec<_> = (0..size)
                        .map(|i| SubmissionEntry::new(OpCode::Fsync, 1, i as u64))
                        .collect();
                    uring.submit_batch(entries).await.unwrap();
                });
            },
        );
    }

    group.finish();
}

fn bench_pipeline_configs(c: &mut Criterion) {
    let mut group = c.benchmark_group("configs");
    group.throughput(Throughput::Elements(1));

    group.bench_function("minimal", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            let uring = NawaUring::new(PipelineConfig::minimal()).unwrap();
            let entry = SubmissionEntry::new(OpCode::Fsync, 1, 0);
            uring.submit(entry).await.unwrap();
        });
    });

    group.bench_function("low_latency", |b| {
        b.to_async(tokio::runtime::Runtime::new().unwrap()).iter(|| async {
            let uring = NawaUring::new(PipelineConfig::low_latency()).unwrap();
            let entry = SubmissionEntry::new(OpCode::Fsync, 1, 0);
            uring.submit(entry).await.unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, bench_submit_single, bench_submit_batch, bench_pipeline_configs);
criterion_main!(benches);
