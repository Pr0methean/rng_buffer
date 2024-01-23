use paste::paste;
use iai::{black_box};
use rand::rngs::adapter::ReseedingRng;
use rand_chacha::{ChaCha12Core};
use rand_core::{OsRng, RngCore, SeedableRng};
use rand_core::block::BlockRng64;
use rng_buffer::RngBufferCore;

const RESEEDING_THRESHOLD: u64 = 1024; // in bytes
const OUTPUT_AMOUNT: u64 = 4096; // in u64's

macro_rules! bench_iai {
    ($n:expr) => {
        paste! {
            pub fn [< buffer_size_ $n >]() {
                let mut buffer = BlockRng64::new(RngBufferCore::<$n, OsRng>(OsRng::default()));
                let mut seed = [0u8; 32];
                buffer.fill_bytes(&mut seed);
                let mut reseeding_from_buffer = ReseedingRng::new(ChaCha12Core::from_seed(seed), RESEEDING_THRESHOLD, buffer);
                (0..OUTPUT_AMOUNT).for_each(|_| {
                    let _ = black_box(reseeding_from_buffer.next_u64());
                })
            }
        }
    }
}

pub fn bench_reseeding_from_os() {
    let mut reseeding_from_os = ReseedingRng::new(
        ChaCha12Core::from_rng(OsRng::default()).unwrap(),
        RESEEDING_THRESHOLD,
        OsRng::default(),
    );
    (0..OUTPUT_AMOUNT).for_each(|_| {
        let _ = black_box(reseeding_from_os.next_u64());
    })
}

bench_iai!(2);
bench_iai!(4);
bench_iai!(8);
bench_iai!(16);