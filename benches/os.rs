use crate::raw::*;
mod raw;

pub fn main() {
    run_in_rayon(bench_reseeding_from_os)
}