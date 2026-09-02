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

//! MurmurHash3 one-shot and streaming APIs.

use core::hash::BuildHasher;
use core::hash::Hasher;

use crate::fmix32;
use crate::read_u32;
use crate::read_u64;

const C1_32: u32 = 0xcc9e_2d51;
const C2_32: u32 = 0x1b87_3593;
const C1_128: u64 = 0x87c3_7b91_1142_53d5;
const C2_128: u64 = 0x4cf5_ad43_2745_937f;

#[inline(always)]
fn mix_k32(value: u32) -> u32 {
    value
        .wrapping_mul(C1_32)
        .rotate_left(15)
        .wrapping_mul(C2_32)
}

#[inline(always)]
fn consume_32(hash: u32, lane: u32) -> u32 {
    (hash ^ mix_k32(lane))
        .rotate_left(13)
        .wrapping_mul(5)
        .wrapping_add(0xe654_6b64)
}

#[inline(always)]
fn finish_32(mut hash: u32, tail: &[u8], total_len: u64) -> u32 {
    let mut lane = 0;
    for (index, &byte) in tail.iter().enumerate() {
        lane |= u32::from(byte) << (index * 8);
    }
    if !tail.is_empty() {
        hash ^= mix_k32(lane);
    }
    fmix32(hash ^ total_len as u32)
}

/// Hashes `input` with the 32-bit x86 variant of MurmurHash3.
#[must_use]
#[inline]
pub fn murmur3_32(input: &[u8], seed: u32) -> u32 {
    let mut hash = seed;
    let mut offset = 0;
    while offset + 4 <= input.len() {
        hash = consume_32(hash, read_u32(input, offset));
        offset += 4;
    }
    finish_32(hash, &input[offset..], input.len() as u64)
}

/// Incremental state for the 32-bit x86 variant of MurmurHash3.
#[derive(Clone, Debug)]
pub struct Murmur3_32 {
    seed: u32,
    hash: u32,
    buffer: [u8; 4],
    buffered: usize,
    total_len: u64,
}

impl Murmur3_32 {
    /// Creates a state with seed zero.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_seed(0)
    }

    /// Creates a state with `seed`.
    #[must_use]
    pub const fn with_seed(seed: u32) -> Self {
        Self {
            seed,
            hash: seed,
            buffer: [0; 4],
            buffered: 0,
            total_len: 0,
        }
    }

    /// Hashes a complete byte slice without constructing streaming state.
    #[must_use]
    #[inline]
    pub fn oneshot(input: &[u8], seed: u32) -> u32 {
        murmur3_32(input, seed)
    }

    /// Adds raw bytes to the hash state.
    pub fn update(&mut self, mut input: &[u8]) {
        self.total_len = self.total_len.wrapping_add(input.len() as u64);

        if self.buffered != 0 {
            let copied = (4 - self.buffered).min(input.len());
            self.buffer[self.buffered..self.buffered + copied].copy_from_slice(&input[..copied]);
            self.buffered += copied;
            input = &input[copied..];
            if self.buffered < 4 {
                return;
            }
            self.hash = consume_32(self.hash, read_u32(&self.buffer, 0));
            self.buffered = 0;
        }

        while input.len() >= 4 {
            self.hash = consume_32(self.hash, read_u32(input, 0));
            input = &input[4..];
        }

        self.buffer[..input.len()].copy_from_slice(input);
        self.buffered = input.len();
    }

    /// Returns the digest without consuming the state.
    #[must_use]
    pub fn digest(&self) -> u32 {
        finish_32(self.hash, &self.buffer[..self.buffered], self.total_len)
    }

    /// Resets the state while retaining its seed.
    pub fn reset(&mut self) {
        *self = Self::with_seed(self.seed);
    }

    /// Returns the seed used by this state.
    #[must_use]
    pub const fn seed(&self) -> u32 {
        self.seed
    }

    /// Returns the number of bytes written so far.
    #[must_use]
    pub const fn total_len(&self) -> u64 {
        self.total_len
    }
}

impl Default for Murmur3_32 {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Murmur3_32 {
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
impl std::io::Write for Murmur3_32 {
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

/// Deterministic [`BuildHasher`] for [`Murmur3_32`].
///
/// MurmurHash3 is intended for trusted inputs and is not resistant to
/// deliberate hash-flooding attacks.
#[derive(Clone, Copy, Debug, Default)]
pub struct Murmur3_32Builder {
    seed: u32,
}

impl Murmur3_32Builder {
    /// Creates a builder using `seed`.
    #[must_use]
    pub const fn with_seed(seed: u32) -> Self {
        Self { seed }
    }
}

impl BuildHasher for Murmur3_32Builder {
    type Hasher = Murmur3_32;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Murmur3_32::with_seed(self.seed)
    }
}

#[inline(always)]
fn mix_k1_128(value: u64) -> u64 {
    value
        .wrapping_mul(C1_128)
        .rotate_left(31)
        .wrapping_mul(C2_128)
}

#[inline(always)]
fn mix_k2_128(value: u64) -> u64 {
    value
        .wrapping_mul(C2_128)
        .rotate_left(33)
        .wrapping_mul(C1_128)
}

#[inline(always)]
fn consume_128(mut hash: [u64; 2], lane1: u64, lane2: u64) -> [u64; 2] {
    hash[0] ^= mix_k1_128(lane1);
    hash[0] = hash[0]
        .rotate_left(27)
        .wrapping_add(hash[1])
        .wrapping_mul(5)
        .wrapping_add(0x52dc_e729);

    hash[1] ^= mix_k2_128(lane2);
    hash[1] = hash[1]
        .rotate_left(31)
        .wrapping_add(hash[0])
        .wrapping_mul(5)
        .wrapping_add(0x3849_5ab5);
    hash
}

#[inline(always)]
fn fmix64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(0xff51_afd7_ed55_8ccd);
    value ^= value >> 33;
    value = value.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    value ^ (value >> 33)
}

#[inline(always)]
fn partial_u64(input: &[u8]) -> u64 {
    let mut value = 0;
    for (index, &byte) in input.iter().enumerate() {
        value |= u64::from(byte) << (index * 8);
    }
    value
}

#[inline(always)]
fn finish_128(mut hash: [u64; 2], tail: &[u8], total_len: u64) -> u128 {
    if tail.len() > 8 {
        hash[1] ^= mix_k2_128(partial_u64(&tail[8..]));
    }
    if !tail.is_empty() {
        hash[0] ^= mix_k1_128(partial_u64(&tail[..tail.len().min(8)]));
    }

    hash[0] ^= total_len;
    hash[1] ^= total_len;
    hash[0] = hash[0].wrapping_add(hash[1]);
    hash[1] = hash[1].wrapping_add(hash[0]);
    hash[0] = fmix64(hash[0]);
    hash[1] = fmix64(hash[1]);
    hash[0] = hash[0].wrapping_add(hash[1]);
    hash[1] = hash[1].wrapping_add(hash[0]);
    (u128::from(hash[1]) << 64) | u128::from(hash[0])
}

/// Hashes `input` with the 128-bit x64 variant of MurmurHash3.
///
/// The returned integer stores the reference algorithm's second 64-bit word
/// in the most significant half and its first word in the least significant
/// half.
#[must_use]
#[inline]
pub fn murmur3_128(input: &[u8], seed: u32) -> u128 {
    let seed = u64::from(seed);
    let mut hash = [seed, seed];
    let mut offset = 0;
    while offset + 16 <= input.len() {
        hash = consume_128(hash, read_u64(input, offset), read_u64(input, offset + 8));
        offset += 16;
    }
    finish_128(hash, &input[offset..], input.len() as u64)
}

/// Explicitly named alias for [`murmur3_128`].
#[must_use]
#[inline]
pub fn murmur3_x64_128(input: &[u8], seed: u32) -> u128 {
    murmur3_128(input, seed)
}

/// Incremental state for the 128-bit x64 variant of MurmurHash3.
///
/// This type does not implement [`Hasher`] because that trait only returns
/// 64-bit digests. Use [`update`](Self::update) for direct incremental input.
/// With the `std` feature, it implements
/// [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html).
#[derive(Clone, Debug)]
pub struct Murmur3_128 {
    seed: u32,
    hash: [u64; 2],
    buffer: [u8; 16],
    buffered: usize,
    total_len: u64,
}

impl Murmur3_128 {
    /// Creates a state with seed zero.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_seed(0)
    }

    /// Creates a state with `seed`.
    #[must_use]
    pub const fn with_seed(seed: u32) -> Self {
        Self {
            seed,
            hash: [seed as u64, seed as u64],
            buffer: [0; 16],
            buffered: 0,
            total_len: 0,
        }
    }

    /// Hashes a complete byte slice without constructing streaming state.
    #[must_use]
    #[inline]
    pub fn oneshot(input: &[u8], seed: u32) -> u128 {
        murmur3_128(input, seed)
    }

    /// Adds raw bytes to the hash state.
    pub fn update(&mut self, mut input: &[u8]) {
        self.total_len = self.total_len.wrapping_add(input.len() as u64);

        if self.buffered != 0 {
            let copied = (16 - self.buffered).min(input.len());
            self.buffer[self.buffered..self.buffered + copied].copy_from_slice(&input[..copied]);
            self.buffered += copied;
            input = &input[copied..];
            if self.buffered < 16 {
                return;
            }
            self.hash = consume_128(
                self.hash,
                read_u64(&self.buffer, 0),
                read_u64(&self.buffer, 8),
            );
            self.buffered = 0;
        }

        while input.len() >= 16 {
            self.hash = consume_128(self.hash, read_u64(input, 0), read_u64(input, 8));
            input = &input[16..];
        }

        self.buffer[..input.len()].copy_from_slice(input);
        self.buffered = input.len();
    }

    /// Returns the digest without consuming the state.
    #[must_use]
    pub fn digest(&self) -> u128 {
        finish_128(self.hash, &self.buffer[..self.buffered], self.total_len)
    }

    /// Alias for [`digest`](Self::digest).
    #[must_use]
    #[inline]
    pub fn finish_128(&self) -> u128 {
        self.digest()
    }

    /// Resets the state while retaining its seed.
    pub fn reset(&mut self) {
        *self = Self::with_seed(self.seed);
    }

    /// Returns the seed used by this state.
    #[must_use]
    pub const fn seed(&self) -> u32 {
        self.seed
    }

    /// Returns the number of bytes written so far.
    #[must_use]
    pub const fn total_len(&self) -> u64 {
        self.total_len
    }
}

impl Default for Murmur3_128 {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "std")]
impl std::io::Write for Murmur3_128 {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_vectors_are_zero_with_zero_seed() {
        assert_eq!(murmur3_32(b"", 0), 0);
        assert_eq!(murmur3_128(b"", 0), 0);
    }

    #[test]
    fn reset_reuses_seed() {
        let mut hash = Murmur3_128::with_seed(42);
        hash.update(b"before");
        hash.reset();
        hash.update(b"after");
        assert_eq!(hash.digest(), murmur3_128(b"after", 42));
    }
}
