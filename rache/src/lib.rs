//! Fast, portable, non-cryptographic hash functions.
//!
//! Use the free functions or [`raw`] module for complete byte slices, and the
//! state types for incremental input. Implementations are grouped under
//! [`cityhash`], [`xxhash`], [`murmur`], and [`fnv`] and re-exported at the crate root.
//! Outputs that fit in 64 bits also implement [`core::hash::Hasher`].
//!
//! These hashes are deterministic and are **not cryptographically secure**.

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(test)]
extern crate std;

pub mod cityhash;
pub mod fnv;
pub mod murmur;
mod util;
pub mod xxhash;

pub use cityhash::{
    cityhash32, cityhash64, cityhash64_with_seed, cityhash64_with_seeds, cityhash128,
    cityhash128_to_64, cityhash128_with_seed,
};
pub use fnv::{Fnv1a32, Fnv1a32Builder, Fnv1a64, Fnv1a64Builder, fnv1a_32, fnv1a_64};
pub use murmur::{
    Murmur3_32, Murmur3_32Builder, Murmur3_128, murmur3_32, murmur3_128, murmur3_x64_128,
};
pub use xxhash::{
    Xxh3, Xxh3_64, Xxh3_128, Xxh3Builder, Xxh32, Xxh32Builder, Xxh64, Xxh64Builder, kernel, xxh3,
    xxh3_64, xxh3_64_with_seed, xxh3_128, xxh3_128_with_seed, xxh32, xxh64,
};

/// Allocation-free one-shot functions for complete byte slices.
///
/// The same functions are also available at the crate root and inside their
/// family modules.
pub mod raw {
    pub use crate::cityhash::{
        cityhash32, cityhash64, cityhash64_with_seed, cityhash64_with_seeds, cityhash128,
        cityhash128_to_64, cityhash128_with_seed,
    };
    pub use crate::fnv::{fnv1a_32, fnv1a_64};
    pub use crate::murmur::{murmur3_32, murmur3_128, murmur3_x64_128};
    pub use crate::xxhash::{
        xxh3_64, xxh3_64_with_seed, xxh3_128, xxh3_128_with_seed, xxh32, xxh64,
    };
}
