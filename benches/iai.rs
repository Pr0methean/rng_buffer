use iai::main;
use crate::raw::*;
mod raw;

main!(
    bench_reseeding_from_os,
    buffer_size_2,
    buffer_size_4,
    buffer_size_8,
    buffer_size_16
);