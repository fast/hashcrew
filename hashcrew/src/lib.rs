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

//! Fast, portable hashing for non-cryptographic use.
//!
//! APIs are grouped by family under [`cityhash`], [`fnv`], [`md5`], [`murmur`],
//! and [`xxhash`]. No features are enabled by default; each family requires its
//! same-named Cargo feature. Use free functions for complete byte slices and
//! state types for incremental input. CityHash is intentionally one-shot.
//! Streaming states with 32- or 64-bit digests also implement [`core::hash::Hasher`].
//!
//! Raw digests are stable across platforms for identical byte streams. The
//! [`core::hash`] adapters use Rust's typed encodings, which can vary across
//! platforms and compiler versions. In particular, hashing a string or slice
//! through [`core::hash::Hash`] can add framing bytes that a raw one-shot call
//! does not receive. Use the free functions or `update` with an explicitly
//! defined byte encoding for persistent checksums and cross-language protocols.
//! These hashes are deterministic and are **not cryptographically secure**.
//!
//! # Choosing an algorithm
//!
//! Prefer XXH3 for new checksums, cache keys, and trusted-input hash tables.
//! The CityHash, MurmurHash3, FNV-1a, XXH32, and XXH64 APIs are primarily for
//! interoperability with an existing format or data set. Choose a 128-bit
//! variant when the application needs a lower collision probability than a
//! 64-bit digest provides.
//! MD5 is available for compatibility with existing formats and protocols that
//! require its standard digest; it is cryptographically broken.
//!
//! # API model
//!
//! Choose the interface from the form of input rather than from a separate
//! implementation:
//!
//! * Call a module-level function such as [`xxhash::xxh3_64`] when the complete byte slice is
//!   available.
//! * Construct a state such as [`xxhash::Xxh3_64`], call its `update` method for each slice, and
//!   call `digest` to read the current result. Further updates extend the same message.
//! * With the `std` feature, use the same state as [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html)
//!   for an I/O producer.
//! * For a Rust hash collection, pass the matching builder as its [`core::hash::BuildHasher`].
//!   These adapters consume Rust's typed [`core::hash::Hash`] encoding rather than a portable byte
//!   serialization.
//!
//! ## Capability map
//!
//! | Variant             | Complete input                               | Incremental state                          | Digest     | [`Hasher`](core::hash::Hasher) / builder                                                          |
//! | ------------------- | -------------------------------------------- | ------------------------------------------ | ---------- | ------------------------------------------------------------------------------------------------- |
//! | CityHash32          | [`cityhash32`](cityhash::cityhash32)         | —                                          | `u32`      | —                                                                                                 |
//! | CityHash64          | [`cityhash64`](cityhash::cityhash64)*        | —                                          | `u64`      | —                                                                                                 |
//! | CityHash128         | [`cityhash128`](cityhash::cityhash128)*      | —                                          | `u128`     | —                                                                                                 |
//! | XXH32               | [`xxh32`](xxhash::xxh32)                     | [`Xxh32`](xxhash::Xxh32)                   | `u32`      | [`Xxh32`](xxhash::Xxh32) / [`Xxh32Builder`](xxhash::Xxh32Builder)                                 |
//! | XXH64               | [`xxh64`](xxhash::xxh64)                     | [`Xxh64`](xxhash::Xxh64)                   | `u64`      | [`Xxh64`](xxhash::Xxh64) / [`Xxh64Builder`](xxhash::Xxh64Builder)                                 |
//! | XXH3-64             | [`xxh3_64`](xxhash::xxh3_64)*                | [`Xxh3_64`](xxhash::Xxh3_64)               | `u64`      | [`Xxh3_64`](xxhash::Xxh3_64) / [`Xxh3_64Builder`](xxhash::Xxh3_64Builder)                         |
//! | XXH3-128            | [`xxh3_128`](xxhash::xxh3_128)*              | [`Xxh3_128`](xxhash::Xxh3_128)             | `u128`     | —                                                                                                 |
//! | MurmurHash3 x86_32  | [`murmur3_x86_32`](murmur::murmur3_x86_32)   | [`Murmur3X86_32`](murmur::Murmur3X86_32)   | `u32`      | [`Murmur3X86_32`](murmur::Murmur3X86_32) / [`Murmur3X86_32Builder`](murmur::Murmur3X86_32Builder) |
//! | MurmurHash3 x86_128 | [`murmur3_x86_128`](murmur::murmur3_x86_128) | [`Murmur3X86_128`](murmur::Murmur3X86_128) | `u128`     | —                                                                                                 |
//! | MurmurHash3 x64_128 | [`murmur3_x64_128`](murmur::murmur3_x64_128) | [`Murmur3X64_128`](murmur::Murmur3X64_128) | `u128`     | —                                                                                                 |
//! | FNV-1a 32           | [`fnv1a_32`](fnv::fnv1a_32)*                 | [`Fnv1a32`](fnv::Fnv1a32)                  | `u32`      | [`Fnv1a32`](fnv::Fnv1a32) / [`Fnv1a32Builder`](fnv::Fnv1a32Builder)                               |
//! | FNV-1a 64           | [`fnv1a_64`](fnv::fnv1a_64)*                 | [`Fnv1a64`](fnv::Fnv1a64)                  | `u64`      | [`Fnv1a64`](fnv::Fnv1a64) / [`Fnv1a64Builder`](fnv::Fnv1a64Builder)                               |
//! | MD5                 | [`md5`](md5::md5)                            | [`Md5`](md5::Md5)                          | `[u8; 16]` | —                                                                                                 |
//!
//! A trailing `*` indicates additional explicitly named configuration forms.
//! [`xxhash::Xxh3_64SecretBuilder`] provides the custom-secret XXH3-64 hash-table
//! adapter. The 128-bit states do not implement [`core::hash::Hasher`] because
//! its [`finish`](core::hash::Hasher::finish) method can only return `u64`.
//! MD5 returns its standard 16 digest bytes; the other 128-bit algorithms return
//! `u128`. Integer results need an explicit byte order for storage or transmission;
//! see each family's module documentation for digest encoding.
//! CityHash has no streaming state because bounded-memory incremental hashing
//! cannot reproduce its one-shot algorithm. MurmurHash3's `x86` and `x64`
//! labels distinguish incompatible algorithms, not target requirements.
//!
//! # Feature flags
//!
//! No features are enabled by default. Enable the families an application uses;
//! each family feature exposes its same-named module:
//!
//! | Feature    | Hash family                              |
//! | ---------- | ---------------------------------------- |
//! | `cityhash` | CityHash32, CityHash64, and CityHash128  |
//! | `fnv`      | FNV-1a 32 and 64                         |
//! | `md5`      | MD5                                      |
//! | `murmur`   | MurmurHash3 x86_32, x86_128, and x64_128 |
//! | `xxhash`   | XXH32, XXH64, XXH3-64, and XXH3-128      |
//!
//! ```toml
//! [dependencies]
//! hashcrew = { version = "0.1", features = ["xxhash", "md5"] }
//! ```
//!
//! All families work without `std`. Enable the independent `std` feature for
//! [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html)
//! adapters and XXH3 runtime CPU-feature detection. It does
//! not enable any hash family. Without it, XXH3 selects hardware kernels only
//! from features guaranteed by the target, with scalar code as the fallback.
//! For example, enable xxHash with standard I/O integration using:
//!
//! ```toml
//! [dependencies]
//! hashcrew = { version = "0.1", features = ["std", "xxhash"] }
//! ```
//!
//! The crate is dependency-free and allocation-free in every configuration.
//! Feature selection does not change digest values.
//!
//! # Streaming input
//!
//! Call `update` when the application already receives byte slices. With the
//! `std` feature, every incremental state can also be the destination of
//! [`std::io::copy`](https://doc.rust-lang.org/std/io/fn.copy.html) or another
//! producer that accepts
//! [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html).
//! Bytes written to the state become hash input: `write` accepts the complete
//! buffer, and `flush` has no work to perform. Obtain the digest separately
//! after the producer finishes.
//!
//! ```
//! # #[cfg(all(feature = "std", feature = "xxhash"))]
//! # {
//! use std::io;
//!
//! use hashcrew::xxhash::Xxh3_64;
//! use hashcrew::xxhash::xxh3_64;
//!
//! let input = b"hashcrew";
//! let mut state = Xxh3_64::new();
//! io::copy(&mut input.as_slice(), &mut state).unwrap();
//!
//! assert_eq!(state.digest(), xxh3_64(input));
//! # }
//! ```
//!
//! # Complete and incremental hashing
//!
//! Hash a complete byte slice with a free function, or feed the same bytes to
//! a reusable state:
//!
//! ```
//! # #[cfg(feature = "xxhash")]
//! # {
//! use hashcrew::xxhash::Xxh64;
//! use hashcrew::xxhash::xxh64;
//!
//! let expected = xxh64(b"hashcrew", 42);
//! let mut state = Xxh64::with_seed(42);
//! state.update(b"hash");
//! state.update(b"crew");
//!
//! assert_eq!(state.digest(), expected);
//! # }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(missing_docs)]
#![no_std]

#[cfg(any(test, feature = "std"))]
extern crate std;

#[cfg(feature = "std")]
mod io;

#[cfg(feature = "cityhash")]
pub mod cityhash;
#[cfg(feature = "fnv")]
pub mod fnv;
#[cfg(feature = "md5")]
pub mod md5;
#[cfg(feature = "murmur")]
pub mod murmur;
#[cfg(feature = "xxhash")]
pub mod xxhash;

#[cfg(any(
    feature = "cityhash",
    feature = "md5",
    feature = "murmur",
    feature = "xxhash"
))]
#[inline(always)]
fn read_u32(input: &[u8], offset: usize) -> u32 {
    let bytes: [u8; 4] = input[offset..offset + 4]
        .try_into()
        .expect("validated hash input range");
    u32::from_le_bytes(bytes)
}

#[cfg(any(feature = "cityhash", feature = "murmur", feature = "xxhash"))]
#[inline(always)]
fn read_u64(input: &[u8], offset: usize) -> u64 {
    let bytes: [u8; 8] = input[offset..offset + 8]
        .try_into()
        .expect("validated hash input range");
    u64::from_le_bytes(bytes)
}

// Ported from Austin Appleby's public-domain MurmurHash3 finalizer:
// https://github.com/aappleby/smhasher/blob/07bb4de10a63e8cc2e1724865454eba635742383/src/MurmurHash3.cpp
#[cfg(any(feature = "cityhash", feature = "murmur"))]
#[inline(always)]
fn fmix32(mut value: u32) -> u32 {
    value ^= value >> 16;
    value = value.wrapping_mul(0x85eb_ca6b);
    value ^= value >> 13;
    value = value.wrapping_mul(0xc2b2_ae35);
    value ^ (value >> 16)
}

// Derived from XXH3_mul128_fold64 in xxHash 0.8.3's xxhash.h:
// https://github.com/Cyan4973/xxHash/blob/e626a72bc2321cd320e953a0ccf1584cad60f363/xxhash.h
// Copyright (C) 2012-2023 Yann Collet. The derived portion remains BSD-2-Clause;
// Hashcrew's modifications are Apache-2.0. See LICENSE for the full upstream terms.
#[cfg(feature = "xxhash")]
#[inline(always)]
fn mul128_fold64(lhs: u64, rhs: u64) -> u64 {
    let product = u128::from(lhs) * u128::from(rhs);
    product as u64 ^ (product >> 64) as u64
}
