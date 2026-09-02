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
//! APIs are grouped by family under [`cityhash`], [`xxhash`], [`murmur`], and
//! [`fnv`]. Use the free functions for complete byte slices and state types for
//! incremental input. CityHash is intentionally one-shot. Outputs that fit in
//! 64 bits also implement [`core::hash::Hasher`].
//!
//! Raw digests are stable across platforms for identical byte streams. The
//! [`core::hash`] adapters use Rust's native typed encodings and are not a
//! portable serialization format. These hashes are deterministic and are
//! **not cryptographically secure**.
//!
//! # Choosing an algorithm
//!
//! Prefer XXH3 for new checksums, cache keys, and trusted-input hash tables.
//! The CityHash, MurmurHash3, FNV-1a, XXH32, and XXH64 APIs are primarily for
//! interoperability with an existing format or data set. Choose a 128-bit
//! variant when the application needs a lower collision probability than a
//! 64-bit digest provides.
//!
//! # Feature flags
//!
//! The crate is dependency-free and allocation-free in every feature
//! configuration. The default `std` feature integrates streaming states with
//! [`std::io`] and enables runtime CPU-feature detection for XXH3. Disable
//! default features for `no_std` targets; hardware kernels are then selected
//! only from features guaranteed by the target, with scalar code as the
//! fallback. Feature selection does not change digest values.
//!
//! ```toml
//! [dependencies]
//! rache = { version = "0.2", default-features = false }
//! ```
//!
//! # Streaming input
//!
//! Call `update` when the application already receives byte slices. With the
//! default `std` feature, every incremental state can also be the destination
//! of [`std::io::copy`] or another producer that writes to [`std::io::Write`].
//! Bytes written to the state become hash input: `write` accepts the complete
//! buffer, and `flush` has no work to perform. Obtain the digest separately
//! after the producer finishes.
//!
//! ```
//! use std::io;
//!
//! use rache::xxhash::Xxh3;
//! use rache::xxhash::xxh3_64;
//!
//! let input = b"rache";
//! let mut state = Xxh3::new();
//! io::copy(&mut input.as_slice(), &mut state).unwrap();
//!
//! assert_eq!(state.digest(), xxh3_64(input));
//! ```
//!
//! # Complete and incremental hashing
//!
//! Hash a complete byte slice with a free function, or feed the same bytes to
//! a reusable state:
//!
//! ```
//! use rache::xxhash::Xxh64;
//! use rache::xxhash::xxh64;
//!
//! let expected = xxh64(b"rache", 42);
//! let mut state = Xxh64::with_seed(42);
//! state.update(b"ra");
//! state.update(b"che");
//!
//! assert_eq!(state.digest(), expected);
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
fn fmix32(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 13;
    value = value.wrapping_mul(0xc2b2_ae35);
    value ^ (value >> 16)
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
