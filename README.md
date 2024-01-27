# rng_buffer

This small crate provides `RngBufferCore`, a struct that wraps any `rand::Rng` and implements `BlockRngCore` so that,
when used in a `rand_core::block::BlockRng64`, it will fetch more bytes with each call. This is mainly intended to
reduce the number of system calls when using `rand::rngs::OsRng` or a client of a remote RNG, for purposes where you
want to reseed regularly to prevent subtle patterns in your random numbers, but don't need fast key erasure (mainly for
Monte Carlo simulations and game servers; I don't recommend it for cryptography or gambling). Profiling with Vtune on an 
EC2 `c7i.metal-24xl` instance running Linux HVM kernel version `6.1.72-96.166.amzn2023.x86_64` showed that this reduced 
the number of CPU cycles spent inside system calls by 80% with a 256-byte buffer per thread (currently the default).

The following are also provided:

* `RngBufferWrapper`, a wrapper around `RngBufferCore` that lets you share the buffer with all of its clones. It also
  bypasses the buffer and directly invokes the wrapped RNG when `fill_bytes` or `try_fill_bytes` is called with a slice
  that's as large as the buffer.
* `RngWrapper`, a struct that wraps any `Rng` in an `Rc<RefCell<_>>` so that clones will use the same instance. Useful
  for implementing custom replacements for `rand::rngs::ThreadRng`.
* `thread_rng()`, a drop-in replacement for `rand::thread_rng()` that uses an `RngBufferCore` per thread but otherwise
  behaves identically.
* `thread_seed_source()`, which provides an `RngBufferWrapper` around a thread-local instance of `OsRng`.
* `build_default_seeder()` and `build_default_rng()`, intended for `no_std` environments where you can't use 
  thread-locals.