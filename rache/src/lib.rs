// Copyright 2026 rache contributors
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Fast, portable, non-cryptographic hash functions.
//!
//! Use the free functions or [`raw`] module for complete byte slices, and the
//! state types for incremental input. CityHash is intentionally one-shot.
//! Implementations are grouped under [`cityhash`], [`xxhash`], [`murmur`], and
//! [`fnv`] and re-exported at the crate root. Outputs that fit in 64 bits also
//! implement [`core::hash::Hasher`].
//!
//! These hashes are deterministic and are **not cryptographically secure**.
//!
//! # Examples
//!
//! Hash a complete byte slice or feed the same bytes incrementally:
//!
//! ```
//! use rache::{Xxh64, raw};
//!
//! let expected = raw::xxh64(b"rache", 42);
//! let mut state = Xxh64::with_seed(42);
//! state.update(b"ra");
//! state.update(b"che");
//!
//! assert_eq!(state.digest(), expected);
//! assert_eq!(raw::cityhash64(b"rache"), rache::cityhash::cityhash64(b"rache"));
//! ```

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
pub use fnv::{
    Fnv1a32, Fnv1a32Builder, Fnv1a64, Fnv1a64Builder, fnv1a_32, fnv1a_32_with_offset_basis,
    fnv1a_64, fnv1a_64_with_offset_basis,
};
pub use murmur::{
    Murmur3_32, Murmur3_32Builder, Murmur3_128, murmur3_32, murmur3_128, murmur3_x64_128,
};
pub use xxhash::{
    DEFAULT_SECRET, DEFAULT_SECRET_SIZE, SECRET_SIZE_MIN, Xxh3, Xxh3_64, Xxh3_128, Xxh3Builder,
    Xxh3SecretBuilder, Xxh3SecretTooShort, Xxh32, Xxh32Builder, Xxh64, Xxh64Builder, kernel, xxh3,
    xxh3_64, xxh3_64_with_secret, xxh3_64_with_seed, xxh3_64_with_seed_and_secret, xxh3_128,
    xxh3_128_with_secret, xxh3_128_with_seed, xxh3_128_with_seed_and_secret, xxh32, xxh64,
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
    pub use crate::fnv::{
        fnv1a_32, fnv1a_32_with_offset_basis, fnv1a_64, fnv1a_64_with_offset_basis,
    };
    pub use crate::murmur::{murmur3_32, murmur3_128, murmur3_x64_128};
    pub use crate::xxhash::{
        xxh3_64, xxh3_64_with_secret, xxh3_64_with_seed, xxh3_64_with_seed_and_secret, xxh3_128,
        xxh3_128_with_secret, xxh3_128_with_seed, xxh3_128_with_seed_and_secret, xxh32, xxh64,
    };
}
