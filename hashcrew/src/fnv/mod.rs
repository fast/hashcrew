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

//! FNV-1a one-shot and streaming APIs with standard or custom offset bases.
//!
//! Call [`fnv1a_32`] or [`fnv1a_64`] for complete input. Use [`Fnv1a32`] or
//! [`Fnv1a64`] when data arrives incrementally; both states also implement
//! [`Hasher`] and have matching [`BuildHasher`] types for trusted-input hash
//! collections. With the default `std` feature, they implement
//! [`std::io::Write`] for I/O producers.
//!
//! A custom offset basis selects a different deterministic output namespace; it
//! is not a security key and does not make FNV resistant to hash flooding.
//!
//! ```
//! use hashcrew::fnv::Fnv1a64;
//! use hashcrew::fnv::fnv1a_64;
//!
//! let mut state = Fnv1a64::new();
//! state.update(b"hash");
//! state.update(b"crew");
//! assert_eq!(state.digest(), fnv1a_64(b"hashcrew"));
//! ```

use core::hash::BuildHasher;
use core::hash::Hasher;

/// Standard 32-bit FNV offset basis.
pub const FNV1A_32_OFFSET_BASIS: u32 = 0x811c_9dc5;
/// Standard 32-bit FNV prime.
pub const FNV1A_32_PRIME: u32 = 0x0100_0193;
/// Standard 64-bit FNV offset basis.
pub const FNV1A_64_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// Standard 64-bit FNV prime.
pub const FNV1A_64_PRIME: u64 = 0x0000_0100_0000_01b3;

#[inline(always)]
fn update_32(mut hash: u32, input: &[u8]) -> u32 {
    for &byte in input {
        hash = (hash ^ u32::from(byte)).wrapping_mul(FNV1A_32_PRIME);
    }
    hash
}

#[inline(always)]
fn update_64(mut hash: u64, input: &[u8]) -> u64 {
    for &byte in input {
        hash = (hash ^ u64::from(byte)).wrapping_mul(FNV1A_64_PRIME);
    }
    hash
}

/// Hashes `input` with 32-bit FNV-1a.
#[must_use]
#[inline]
pub fn fnv1a_32(input: &[u8]) -> u32 {
    fnv1a_32_with_offset_basis(input, FNV1A_32_OFFSET_BASIS)
}

/// Hashes `input` with 32-bit FNV-1a starting from `offset_basis`.
///
/// Digests produced with a non-standard offset basis do not interoperate with
/// standard FNV-1a digests.
#[must_use]
#[inline]
pub fn fnv1a_32_with_offset_basis(input: &[u8], offset_basis: u32) -> u32 {
    update_32(offset_basis, input)
}

/// Hashes `input` with 64-bit FNV-1a.
#[must_use]
#[inline]
pub fn fnv1a_64(input: &[u8]) -> u64 {
    fnv1a_64_with_offset_basis(input, FNV1A_64_OFFSET_BASIS)
}

/// Hashes `input` with 64-bit FNV-1a starting from `offset_basis`.
///
/// Digests produced with a non-standard offset basis do not interoperate with
/// standard FNV-1a digests.
#[must_use]
#[inline]
pub fn fnv1a_64_with_offset_basis(input: &[u8], offset_basis: u64) -> u64 {
    update_64(offset_basis, input)
}

/// Incremental 32-bit FNV-1a state.
#[derive(Clone, Copy, Debug)]
pub struct Fnv1a32 {
    hash: u32,
    offset_basis: u32,
}

impl Fnv1a32 {
    /// Creates a state using the standard offset basis.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_offset_basis(FNV1A_32_OFFSET_BASIS)
    }

    /// Creates a state using `offset_basis`.
    #[must_use]
    pub const fn with_offset_basis(offset_basis: u32) -> Self {
        Self {
            hash: offset_basis,
            offset_basis,
        }
    }

    /// Adds raw bytes to the hash state.
    #[inline]
    pub fn update(&mut self, input: &[u8]) {
        self.hash = update_32(self.hash, input);
    }

    /// Returns the digest without consuming the state.
    #[must_use]
    pub const fn digest(&self) -> u32 {
        self.hash
    }

    /// Returns the offset basis retained by this state.
    #[must_use]
    pub const fn offset_basis(&self) -> u32 {
        self.offset_basis
    }

    /// Resets the state to its configured offset basis.
    pub fn reset(&mut self) {
        self.hash = self.offset_basis;
    }
}

impl Default for Fnv1a32 {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Fnv1a32 {
    #[inline]
    fn finish(&self) -> u64 {
        u64::from(self.digest())
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }
}

#[cfg(feature = "std")]
impl std::io::Write for Fnv1a32 {
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

/// Deterministic [`BuildHasher`] for [`Fnv1a32`].
///
/// FNV-1a is intended for trusted inputs and is not resistant to deliberate
/// hash-flooding attacks.
#[derive(Clone, Copy, Debug)]
pub struct Fnv1a32Builder {
    offset_basis: u32,
}

impl Fnv1a32Builder {
    /// Creates a builder using `offset_basis`.
    #[must_use]
    pub const fn with_offset_basis(offset_basis: u32) -> Self {
        Self { offset_basis }
    }
}

impl Default for Fnv1a32Builder {
    fn default() -> Self {
        Self::with_offset_basis(FNV1A_32_OFFSET_BASIS)
    }
}

impl BuildHasher for Fnv1a32Builder {
    type Hasher = Fnv1a32;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Fnv1a32::with_offset_basis(self.offset_basis)
    }
}

/// Incremental 64-bit FNV-1a state.
#[derive(Clone, Copy, Debug)]
pub struct Fnv1a64 {
    hash: u64,
    offset_basis: u64,
}

impl Fnv1a64 {
    /// Creates a state using the standard offset basis.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_offset_basis(FNV1A_64_OFFSET_BASIS)
    }

    /// Creates a state using `offset_basis`.
    #[must_use]
    pub const fn with_offset_basis(offset_basis: u64) -> Self {
        Self {
            hash: offset_basis,
            offset_basis,
        }
    }

    /// Adds raw bytes to the hash state.
    #[inline]
    pub fn update(&mut self, input: &[u8]) {
        self.hash = update_64(self.hash, input);
    }

    /// Returns the digest without consuming the state.
    #[must_use]
    pub const fn digest(&self) -> u64 {
        self.hash
    }

    /// Returns the offset basis retained by this state.
    #[must_use]
    pub const fn offset_basis(&self) -> u64 {
        self.offset_basis
    }

    /// Resets the state to its configured offset basis.
    pub fn reset(&mut self) {
        self.hash = self.offset_basis;
    }
}

impl Default for Fnv1a64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Fnv1a64 {
    #[inline]
    fn finish(&self) -> u64 {
        self.digest()
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }
}

#[cfg(feature = "std")]
impl std::io::Write for Fnv1a64 {
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

/// Deterministic [`BuildHasher`] for [`Fnv1a64`].
///
/// FNV-1a is intended for trusted inputs and is not resistant to deliberate
/// hash-flooding attacks.
#[derive(Clone, Copy, Debug)]
pub struct Fnv1a64Builder {
    offset_basis: u64,
}

impl Fnv1a64Builder {
    /// Creates a builder using `offset_basis`.
    #[must_use]
    pub const fn with_offset_basis(offset_basis: u64) -> Self {
        Self { offset_basis }
    }
}

impl Default for Fnv1a64Builder {
    fn default() -> Self {
        Self::with_offset_basis(FNV1A_64_OFFSET_BASIS)
    }
}

impl BuildHasher for Fnv1a64Builder {
    type Hasher = Fnv1a64;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Fnv1a64::with_offset_basis(self.offset_basis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specification_vectors_match() {
        let vectors = [
            (b"".as_slice(), 0x811c_9dc5, 0xcbf2_9ce4_8422_2325),
            (b"a".as_slice(), 0xe40c_292c, 0xaf63_dc4c_8601_ec8c),
            (b"foobar".as_slice(), 0xbf9c_f968, 0x8594_4171_f739_67e8),
        ];
        for (input, expected32, expected64) in vectors {
            assert_eq!(fnv1a_32(input), expected32);
            assert_eq!(fnv1a_64(input), expected64);
        }
    }

    #[test]
    fn reset_restores_offset_basis() {
        let mut hash = Fnv1a64::new();
        hash.update(b"before");
        hash.reset();
        hash.update(b"after");
        assert_eq!(hash.digest(), fnv1a_64(b"after"));
    }
}
