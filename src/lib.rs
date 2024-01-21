use bytemuck::cast_slice_mut;
use std::cell::{RefCell};
use std::rc::Rc;
use rand::rngs::adapter::ReseedingRng;
use rand_chacha::ChaCha12Core;
use rand_core::{Error, RngCore, SeedableRng};
use rand_core::block::{BlockRng64, BlockRngCore};
use rand_core::OsRng;

#[derive(Copy, Clone)]
pub struct DefaultableArray<const N: usize, T>([T; N]);

impl <const N: usize, T: Default + Copy> Default for DefaultableArray<N, T> {
    fn default() -> Self {
        Self([T::default(); N])
    }
}

impl<const N: usize, T> AsMut<[T; N]> for DefaultableArray<N, T> {
    fn as_mut(&mut self) -> &mut [T; N] {
        &mut self.0
    }
}

impl<const N: usize, T> AsRef<[T; N]> for DefaultableArray<N, T> {
    fn as_ref(&self) -> &[T; N] {
        &self.0
    }
}

impl<const N: usize, T> AsRef<[T]> for DefaultableArray<N, T> {
    fn as_ref(&self) -> &[T] {
        self.0.as_slice()
    }
}

impl<const N: usize, T> AsMut<[T]> for DefaultableArray<N, T> {
    fn as_mut(&mut self) -> &mut [T] {
        self.0.as_mut_slice()
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RngBufferCore<const N: usize, T: RngCore>(pub T);

const WORDS_PER_STD_RNG_SEED: usize = 4;
const DEFAULT_SEEDS_PER_BUFFER: usize = 16;
const DEFAULT_BUFFER_SIZE: usize = WORDS_PER_STD_RNG_SEED * DEFAULT_SEEDS_PER_BUFFER;
const DEFAULT_RESEEDING_THRESHOLD: u64 = 1 << 16;

impl <const N: usize, T: RngCore> BlockRngCore for RngBufferCore<N, T> {
    type Item = u64;
    type Results = DefaultableArray<N, u64>;

    fn generate(&mut self, results: &mut Self::Results) {
        self.0.fill_bytes(cast_slice_mut(results.as_mut()));
    }
}

#[derive(Clone)]
pub struct RngCoreWrapper<T: RngCore>(Rc<RefCell<T>>);

impl <T: RngCore> RngCore for RngCoreWrapper<T> {
    fn next_u32(&mut self) -> u32 {
        self.0.as_ref().borrow_mut().next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.0.as_ref().borrow_mut().next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.0.as_ref().borrow_mut().fill_bytes(dest)
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
        self.0.as_ref().borrow_mut().try_fill_bytes(dest)
    }
}

pub type DefaultRngBufferCore = RngBufferCore<DEFAULT_BUFFER_SIZE, OsRng>;

pub type DefaultSeedSourceRng = RngCoreWrapper<BlockRng64<DefaultRngBufferCore>>;

pub type DefaultRng = ReseedingRng<ChaCha12Core, DefaultSeedSourceRng>;

thread_local! {
    static DEFAULT_INSTANCES: DefaultSeedSourceRng
        = RngCoreWrapper(Rc::new(RefCell::new(BlockRng64::new(RngBufferCore(OsRng::default())))));
}

pub fn thread_seed_source() -> DefaultSeedSourceRng {
    DEFAULT_INSTANCES.with(RngCoreWrapper::clone)
}

pub fn thread_rng() -> DefaultRng {
    DEFAULT_INSTANCES.with(|seed_source| {
        let mut seed = [0u8; 32];
        OsRng::default().fill_bytes(&mut seed);
        ReseedingRng::new(ChaCha12Core::from_seed(seed), DEFAULT_RESEEDING_THRESHOLD,
                          seed_source.clone())
    })
}

#[cfg(test)]
mod tests {
    use rand_core::{Error, OsRng};
    use rand_core::block::{BlockRng64};
    use crate::{DefaultRngBufferCore, RngBufferCore};

    #[test]
    fn basic_test() -> Result<(), Error> {
        use rand::rngs::StdRng;
        use rand::SeedableRng;
        let shared_seeder: DefaultRngBufferCore = RngBufferCore(OsRng::default());
        let client_prng: StdRng = StdRng::from_rng(&mut BlockRng64::new(shared_seeder))?;
        let zero_seed_prng = StdRng::from_seed([0; 32]);
        assert_ne!(client_prng, zero_seed_prng);
        Ok(())
    }
}