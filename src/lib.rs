#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::rc::Rc;
use bytemuck::cast_slice_mut;
use core::cell::RefCell;
use std::intrinsics::size_of;
use delegate::delegate;
use rand::rngs::adapter::ReseedingRng;
use rand_chacha::ChaCha12Core;
use rand_core::{Error, OsRng, RngCore, SeedableRng};
use rand_core::block::{BlockRng64, BlockRngCore};

/// Wrapper around an array, that can implement [Default].
#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct DefaultableArray<const N: usize, T: Default + Copy>([T; N]);

impl <const N: usize, T: Default + Copy> Default for DefaultableArray<N, T> {
    fn default() -> Self {
        Self([T::default(); N])
    }
}

impl<const N: usize, T: Default + Copy> AsMut<[T; N]> for DefaultableArray<N, T> {
    fn as_mut(&mut self) -> &mut [T; N] {
        &mut self.0
    }
}

impl<const N: usize, T: Default + Copy> AsRef<[T; N]> for DefaultableArray<N, T> {
    fn as_ref(&self) -> &[T; N] {
        &self.0
    }
}

impl<const N: usize, T: Default + Copy> AsRef<[T]> for DefaultableArray<N, T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_slice()
    }
}

impl<const N: usize, T: Default + Copy> AsMut<[T]> for DefaultableArray<N, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(transparent)]
pub struct RngBufferCore<const N: usize, T: RngCore>(pub T);

const WORDS_PER_STD_RNG_SEED: usize = 4;
const DEFAULT_SEEDS_PER_BUFFER: usize = 16;
const DEFAULT_BUFFER_SIZE: usize = WORDS_PER_STD_RNG_SEED * DEFAULT_SEEDS_PER_BUFFER;

impl <const N: usize, T: RngCore> BlockRngCore for RngBufferCore<N, T> {
    type Item = u64;
    type Results = DefaultableArray<N, u64>;

    fn generate(&mut self, results: &mut Self::Results) {
        self.0.fill_bytes(cast_slice_mut(results.as_mut()));
    }
}

impl <const N: usize, T: RngCore> From<T> for RngBufferCore<N, T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

/// Wraps an RNG in a buffering [BlockRng64], and also in an [Rc] and [RefCell] so that the buffer can be shared (within
/// the same thread) with all clones of this instance.
#[derive(Clone)]
#[repr(transparent)]
pub struct RngBufferWrapper<const N: usize, T: RngCore>(Rc<RefCell<BlockRng64<RngBufferCore<N, T>>>>);

impl <const N: usize, T: RngCore> From<T> for RngBufferWrapper<N, T> {
    fn from(value: T) -> Self {
        Self(Rc::new(RefCell::new(BlockRng64::new(value.into()))))
    }
}

/// Wraps an RNG in an [Rc] and [RefCell] so that it can be shared (within the same thread) across structs that expect
/// to own one.
#[derive(Clone)]
#[repr(transparent)]
pub struct RngWrapper<T: RngCore>(Rc<RefCell<T>>);

impl <T: RngCore> From<T> for RngWrapper<T> {
    fn from(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
}

impl <const N: usize, T: RngCore> RngCore for RngBufferWrapper<N, T> {
    delegate!{
        to self.0.as_ref().borrow_mut().core.0 {
            fn next_u32(&mut self) -> u32;
            fn next_u64(&mut self) -> u64;
        }
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.try_fill_bytes(dest).expect("RngBufferWrapper core threw an error from try_fill_bytes")
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        if dest.len() > N * size_of::<u64>() {
            unsafe { self.0.as_ref().as_ptr().as_mut().unwrap().core.0.try_fill_bytes(dest) }
        } else {
            self.0.as_ptr().try_fill_bytes(dest)
        }
    }
}


impl <T: RngCore> RngCore for RngWrapper<T> {
    delegate!{
        to self.0.borrow_mut() {
            fn next_u32(&mut self) -> u32;
            fn next_u64(&mut self) -> u64;
            fn fill_bytes(&mut self, dest: &mut [u8]);
            fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error>;
        }
    }
}

pub type DefaultSeedSourceRng = RngBufferWrapper<DEFAULT_BUFFER_SIZE, OsRng>;

pub fn build_default_seeder() -> DefaultSeedSourceRng {
   OsRng::default().into()
}

impl Default for DefaultSeedSourceRng {
    #[cfg(feature = "std")]
    fn default() -> Self {
        thread_seed_source()
    }
    #[cfg(not(feature = "std"))]
    fn default() -> Self {
        build_default_seeder()
    }
}

pub type DefaultRng = RngWrapper<ReseedingRng<ChaCha12Core, DefaultSeedSourceRng>>;

pub fn build_default_rng(mut seeder: DefaultSeedSourceRng) -> DefaultRng {
    let mut seed = [0u8; 32];
    seeder.fill_bytes(&mut seed);
    ReseedingRng::new(ChaCha12Core::from_seed(seed), THREAD_RNG_RESEED_THRESHOLD, seeder).into()
}

impl Default for DefaultRng {
    #[cfg(feature = "std")]
    fn default() -> Self {
        thread_rng()
    }

    #[cfg(not(feature = "std"))]
    fn default() -> Self {
        build_default_rng(DefaultSeedSourceRng::default())
    }
}

const THREAD_RNG_RESEED_THRESHOLD: u64 = 1 << 16;

#[cfg(feature = "std")]
thread_local! {
    static THREAD_SEEDER_KEY: DefaultSeedSourceRng = build_default_seeder();
    static THREAD_RNG_KEY: DefaultRng = THREAD_SEEDER_KEY.with(|seeder| {
            build_default_rng(seeder.clone())
        });
}

/// Obtains this thread's default buffering wrapper around [OsRng]. Produces the same output as [OsRng], but with the
/// ability to buffer output from one system call and use it to fulfill multiple requests.
#[cfg(feature = "std")]
pub fn thread_seed_source() -> DefaultSeedSourceRng {
    THREAD_SEEDER_KEY.with(DefaultSeedSourceRng::clone)
}

/// Obtains this thread's default RNG, which is identical to [rand::thread_rng]() except that it uses
/// [thread_seed_source]() to reseed itself rather than directly calling [OsRng].
#[cfg(feature = "std")]
pub fn thread_rng() -> DefaultRng {
    THREAD_RNG_KEY.with(DefaultRng::clone)
}

#[cfg(test)]
mod tests {
    use rand_core::{Error};
    use crate::{build_default_seeder, DefaultSeedSourceRng};

    #[test]
    fn basic_test() -> Result<(), Error> {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let shared_seeder: DefaultSeedSourceRng = build_default_seeder();
        let client_prng: StdRng = StdRng::from_rng(shared_seeder)?;
        let zero_seed_prng = StdRng::from_seed([0; 32]);
        assert_ne!(client_prng, zero_seed_prng);
        Ok(())
    }
}