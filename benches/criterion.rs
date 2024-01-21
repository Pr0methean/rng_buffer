use std::mem::size_of;
use criterion::{black_box, BenchmarkId, Criterion, criterion_group, criterion_main, Throughput};
use rand::rngs::adapter::ReseedingRng;
use rand_chacha::ChaCha12Core;
use rand_core::{OsRng, RngCore, SeedableRng};
use rand_core::block::BlockRng64;
use crate::RngBufferCore;

const RESEEDING_THRESHOLD: u64 = 1024;

macro_rules! bench {
    ($group:expr, $n:expr) => {
        let mut buffer = BlockRng64::new(RngBufferCore::<$n, OsRng>(OsRng::default()));
        let mut seed = [0u8; 32];
        buffer.fill_bytes(&mut seed);
        let mut reseeding_from_buffer = ReseedingRng::new(ChaCha12Core::from_seed(seed), RESEEDING_THRESHOLD, buffer);
        $group.bench_with_input(BenchmarkId::new("RngBufferCore", $n),
        &$n, |b, _| b.iter(|| black_box(reseeding_from_buffer.next_u64())));
    };
}

fn benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("Single Thread");
    group.throughput(Throughput::Bytes(size_of::<u64>() as u64));
    bench!(group, 2);
    bench!(group, 4);
    bench!(group, 8);
    bench!(group, 16);
    bench!(group, 32);
    bench!(group, 64);
    bench!(group, 128);
    bench!(group, 256);
    bench!(group, 512);
    bench!(group, 1024);
    let mut reseeding_from_os = ReseedingRng::new(
        ChaCha12Core::from_rng(OsRng::default()).unwrap(),
        RESEEDING_THRESHOLD,
        OsRng::default(),
    );
    group.bench_function("OsRng", |b| {
        b.iter(|| black_box(reseeding_from_os.next_u64()))
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().confidence_level(0.99).sample_size(4096);
    targets = benchmark
}
criterion_main!(benches);