//! Fast, non-cryptographic hash functions.
//!
//! `rache` currently implements the xxHash family: XXH32, XXH64,
//! XXH3-64, and XXH3-128. The free functions are the fastest API when all
//! bytes are already available. Streaming hashers are provided for incremental
//! input and implement [`core::hash::Hasher`] where the output fits in 64 bits.
//!
//! These hashes are deterministic and are **not cryptographically secure**.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(test)]
extern crate std;

pub mod kernel;
mod util;
pub mod xxh3;
pub mod xxh32;
pub mod xxh64;

pub use xxh3::{
    Xxh3, Xxh3_64, Xxh3_128, Xxh3Builder, xxh3_64, xxh3_64_with_seed, xxh3_128, xxh3_128_with_seed,
};
pub use xxh32::{Xxh32, Xxh32Builder, xxh32};
pub use xxh64::{Xxh64, Xxh64Builder, xxh64};

/// Allocation-free, one-shot hashing functions.
///
/// This module exists as a compact import target for callers that only hash raw
/// byte slices. The same functions are also available at the crate root.
pub mod raw {
    pub use crate::xxh3::{xxh3_64, xxh3_64_with_seed, xxh3_128, xxh3_128_with_seed};
    pub use crate::xxh32::xxh32;
    pub use crate::xxh64::xxh64;
}
