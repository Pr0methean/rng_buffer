use paste::paste;
use iai::{black_box, main};
use rand::rngs::adapter::ReseedingRng;
use rand_chacha::{ChaCha12Core};
use rand_core::{OsRng, RngCore, SeedableRng};
use rand_core::block::BlockRng64;
use rng_buffer::RngBufferCore;

const RESEEDING_THRESHOLD: u64 = 1024;

macro_rules! bench_iai {
    ($n:expr) => {
        paste! {
            fn [< bench_iai_buffer_size_ $n >]() {
                let mut buffer = BlockRng64::new(RngBufferCore::<$n, OsRng>(OsRng::default()));
                let mut seed = [0u8; 32];
                buffer.fill_bytes(&mut seed);
                let mut reseeding_from_buffer = ReseedingRng::new(ChaCha12Core::from_seed(seed), RESEEDING_THRESHOLD, buffer);
                (0..(2 * RESEEDING_THRESHOLD * $n.max(1))).for_each(|_| {
                    let _ = black_box(reseeding_from_buffer.next_u64());
                })
            }
        }
    }
}

bench_iai!(2);
bench_iai!(4);
bench_iai!(8);
bench_iai!(16);

main!(
    bench_iai_buffer_size_2,
    bench_iai_buffer_size_4,
    bench_iai_buffer_size_8,
    bench_iai_buffer_size_16
);