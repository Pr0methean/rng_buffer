use crate::raw::*;
mod raw;

pub fn main() {
    run_in_rayon(buffer_size_8)
}