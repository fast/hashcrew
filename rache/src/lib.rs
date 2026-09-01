// Copyright 2026 FastLabs Developers
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
//! Use the [`raw`] module for complete byte slices, and the family modules for
//! state types and family-specific APIs. CityHash is intentionally one-shot.
//! Outputs that fit in 64 bits also implement [`core::hash::Hasher`]. With the
//! default `std` feature, every incremental state also implements
//! [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html).
//!
//! Raw digests are stable across platforms for identical byte streams. The
//! [`core::hash`] adapters use Rust's native typed encodings and are not a
//! portable serialization format. These hashes are deterministic and are
//! **not cryptographically secure**.
//!
//! # Examples
//!
//! Hash a complete byte slice or feed the same bytes incrementally:
//!
//! ```
//! use rache::raw;
//! use rache::xxhash::Xxh64;
//!
//! let expected = raw::xxh64(b"rache", 42);
//! let mut state = Xxh64::with_seed(42);
//! state.update(b"ra");
//! state.update(b"che");
//!
//! assert_eq!(state.digest(), expected);
//! assert_eq!(
//!     raw::cityhash64(b"rache"),
//!     rache::cityhash::cityhash64(b"rache")
//! );
//! ```

#![cfg_attr(not(feature = "std"), no_std)]
#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(rust_2018_idioms)]
#![forbid(unsafe_op_in_unsafe_fn)]

#[cfg(test)]
extern crate std;

#[inline(always)]
fn read_u32(input: &[u8], offset: usize) -> u32 {
    let bytes: [u8; 4] = input[offset..offset + 4]
        .try_into()
        .expect("validated hash input range");
    u32::from_le_bytes(bytes)
}

#[inline(always)]
fn read_u64(input: &[u8], offset: usize) -> u64 {
    let bytes: [u8; 8] = input[offset..offset + 8]
        .try_into()
        .expect("validated hash input range");
    u64::from_le_bytes(bytes)
}

#[inline(always)]
fn mul128_fold64(lhs: u64, rhs: u64) -> u64 {
    let product = u128::from(lhs) * u128::from(rhs);
    product as u64 ^ (product >> 64) as u64
}

pub mod cityhash;
pub mod fnv;
pub mod murmur;
pub mod xxhash;

/// Allocation-free one-shot functions for complete byte slices.
///
/// The same functions are also available inside their family modules.
pub mod raw {
    pub use crate::cityhash::cityhash32;
    pub use crate::cityhash::cityhash64;
    pub use crate::cityhash::cityhash64_with_seed;
    pub use crate::cityhash::cityhash64_with_seeds;
    pub use crate::cityhash::cityhash128;
    pub use crate::cityhash::cityhash128_to_64;
    pub use crate::cityhash::cityhash128_with_seed;
    pub use crate::fnv::fnv1a_32;
    pub use crate::fnv::fnv1a_32_with_offset_basis;
    pub use crate::fnv::fnv1a_64;
    pub use crate::fnv::fnv1a_64_with_offset_basis;
    pub use crate::murmur::murmur3_32;
    pub use crate::murmur::murmur3_128;
    pub use crate::murmur::murmur3_x64_128;
    pub use crate::xxhash::xxh3_64;
    pub use crate::xxhash::xxh3_64_with_secret;
    pub use crate::xxhash::xxh3_64_with_seed;
    pub use crate::xxhash::xxh3_64_with_seed_and_secret;
    pub use crate::xxhash::xxh3_128;
    pub use crate::xxhash::xxh3_128_with_secret;
    pub use crate::xxhash::xxh3_128_with_seed;
    pub use crate::xxhash::xxh3_128_with_seed_and_secret;
    pub use crate::xxhash::xxh32;
    pub use crate::xxhash::xxh64;
}
