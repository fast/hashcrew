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
//! Use the free functions or [`raw`] module for complete byte slices, and the
//! state types for incremental input. CityHash is intentionally one-shot.
//! Implementations are grouped under [`cityhash`], [`xxhash`], [`murmur`], and
//! [`fnv`] and re-exported at the crate root. Outputs that fit in 64 bits also
//! implement [`core::hash::Hasher`]. With the default `std` feature, every
//! incremental state also implements
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
//! use rache::Xxh64;
//! use rache::raw;
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

macro_rules! impl_std_io_write {
    ($hash:ty) => {
        #[cfg(feature = "std")]
        impl std::io::Write for $hash {
            #[inline]
            fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
                self.update(input);
                Ok(input.len())
            }

            #[inline]
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
    };
    ($hash:ty, $storage:ident) => {
        #[cfg(feature = "std")]
        impl<$storage: AsRef<[u8]>> std::io::Write for $hash {
            #[inline]
            fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
                self.update(input);
                Ok(input.len())
            }

            #[inline]
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
    };
}

pub mod cityhash;
pub mod fnv;
pub mod murmur;
mod util;
pub mod xxhash;

pub use cityhash::cityhash32;
pub use cityhash::cityhash64;
pub use cityhash::cityhash64_with_seed;
pub use cityhash::cityhash64_with_seeds;
pub use cityhash::cityhash128;
pub use cityhash::cityhash128_to_64;
pub use cityhash::cityhash128_with_seed;
pub use fnv::Fnv1a32;
pub use fnv::Fnv1a32Builder;
pub use fnv::Fnv1a64;
pub use fnv::Fnv1a64Builder;
pub use fnv::fnv1a_32;
pub use fnv::fnv1a_32_with_offset_basis;
pub use fnv::fnv1a_64;
pub use fnv::fnv1a_64_with_offset_basis;
pub use murmur::Murmur3_32;
pub use murmur::Murmur3_32Builder;
pub use murmur::Murmur3_128;
pub use murmur::murmur3_32;
pub use murmur::murmur3_128;
pub use murmur::murmur3_x64_128;
pub use xxhash::DEFAULT_SECRET;
pub use xxhash::DEFAULT_SECRET_SIZE;
pub use xxhash::SECRET_SIZE_MIN;
pub use xxhash::Xxh3;
pub use xxhash::Xxh3_64;
pub use xxhash::Xxh3_128;
pub use xxhash::Xxh3Builder;
pub use xxhash::Xxh3SecretBuilder;
pub use xxhash::Xxh3SecretTooShort;
pub use xxhash::Xxh32;
pub use xxhash::Xxh32Builder;
pub use xxhash::Xxh64;
pub use xxhash::Xxh64Builder;
pub use xxhash::kernel;
pub use xxhash::xxh3;
pub use xxhash::xxh3_64;
pub use xxhash::xxh3_64_with_secret;
pub use xxhash::xxh3_64_with_seed;
pub use xxhash::xxh3_64_with_seed_and_secret;
pub use xxhash::xxh3_128;
pub use xxhash::xxh3_128_with_secret;
pub use xxhash::xxh3_128_with_seed;
pub use xxhash::xxh3_128_with_seed_and_secret;
pub use xxhash::xxh32;
pub use xxhash::xxh64;

/// Allocation-free one-shot functions for complete byte slices.
///
/// The same functions are also available at the crate root and inside their
/// family modules.
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
