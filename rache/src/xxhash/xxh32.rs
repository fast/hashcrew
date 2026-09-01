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

//! XXH32 one-shot and streaming APIs.

use core::hash::{BuildHasher, Hasher};

use crate::util::read_u32;

const PRIME1: u32 = 0x9e37_79b1;
const PRIME2: u32 = 0x85eb_ca77;
const PRIME3: u32 = 0xc2b2_ae3d;
const PRIME4: u32 = 0x27d4_eb2f;
const PRIME5: u32 = 0x1656_67b1;

#[inline(always)]
fn round(acc: u32, lane: u32) -> u32 {
    acc.wrapping_add(lane.wrapping_mul(PRIME2))
        .rotate_left(13)
        .wrapping_mul(PRIME1)
}

#[inline(always)]
fn avalanche(mut hash: u32) -> u32 {
    hash ^= hash >> 15;
    hash = hash.wrapping_mul(PRIME2);
    hash ^= hash >> 13;
    hash = hash.wrapping_mul(PRIME3);
    hash ^ (hash >> 16)
}

#[inline(always)]
fn consume_tail(mut hash: u32, tail: &[u8]) -> u32 {
    let mut offset = 0;
    while offset + 4 <= tail.len() {
        hash = hash.wrapping_add(read_u32(tail, offset).wrapping_mul(PRIME3));
        hash = hash.rotate_left(17).wrapping_mul(PRIME4);
        offset += 4;
    }
    for &byte in &tail[offset..] {
        hash = hash.wrapping_add(u32::from(byte).wrapping_mul(PRIME5));
        hash = hash.rotate_left(11).wrapping_mul(PRIME1);
    }
    avalanche(hash)
}

/// Hashes `input` with XXH32 and `seed`.
#[must_use]
#[inline]
pub fn xxh32(input: &[u8], seed: u32) -> u32 {
    let mut offset = 0;
    let mut lanes = [
        seed.wrapping_add(PRIME1).wrapping_add(PRIME2),
        seed.wrapping_add(PRIME2),
        seed,
        seed.wrapping_sub(PRIME1),
    ];

    while offset + 16 <= input.len() {
        lanes[0] = round(lanes[0], read_u32(input, offset));
        lanes[1] = round(lanes[1], read_u32(input, offset + 4));
        lanes[2] = round(lanes[2], read_u32(input, offset + 8));
        lanes[3] = round(lanes[3], read_u32(input, offset + 12));
        offset += 16;
    }

    let mut hash = if input.len() >= 16 {
        lanes[0]
            .rotate_left(1)
            .wrapping_add(lanes[1].rotate_left(7))
            .wrapping_add(lanes[2].rotate_left(12))
            .wrapping_add(lanes[3].rotate_left(18))
    } else {
        seed.wrapping_add(PRIME5)
    };
    hash = hash.wrapping_add(input.len() as u32);
    consume_tail(hash, &input[offset..])
}

/// Incremental XXH32 state.
#[derive(Clone, Debug)]
pub struct Xxh32 {
    seed: u32,
    lanes: [u32; 4],
    buffer: [u8; 16],
    buffered: usize,
    total_len: u64,
    length_overflowed: bool,
}

impl Xxh32 {
    /// Creates an unseeded XXH32 state.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_seed(0)
    }

    /// Hashes a complete byte slice without constructing streaming state.
    #[must_use]
    #[inline]
    pub fn oneshot(input: &[u8], seed: u32) -> u32 {
        xxh32(input, seed)
    }

    /// Creates an XXH32 state with `seed`.
    #[must_use]
    pub const fn with_seed(seed: u32) -> Self {
        Self {
            seed,
            lanes: [
                seed.wrapping_add(PRIME1).wrapping_add(PRIME2),
                seed.wrapping_add(PRIME2),
                seed,
                seed.wrapping_sub(PRIME1),
            ],
            buffer: [0; 16],
            buffered: 0,
            total_len: 0,
            length_overflowed: false,
        }
    }

    /// Returns the seed used by this state.
    #[must_use]
    pub const fn seed(&self) -> u32 {
        self.seed
    }

    /// Returns the number of bytes written so far, saturating at [`u64::MAX`].
    #[must_use]
    pub const fn total_len(&self) -> u64 {
        if self.length_overflowed {
            u64::MAX
        } else {
            self.total_len
        }
    }

    /// Resets the state while retaining its seed.
    pub fn reset(&mut self) {
        *self = Self::with_seed(self.seed);
    }

    /// Adds raw bytes to the hash state.
    pub fn update(&mut self, mut input: &[u8]) {
        let (total_len, overflowed) = self.total_len.overflowing_add(input.len() as u64);
        self.total_len = total_len;
        self.length_overflowed |= overflowed;

        if self.buffered != 0 {
            let needed = 16 - self.buffered;
            let copied = needed.min(input.len());
            self.buffer[self.buffered..self.buffered + copied].copy_from_slice(&input[..copied]);
            self.buffered += copied;
            input = &input[copied..];
            if self.buffered < 16 {
                return;
            }
            consume_block(&mut self.lanes, &self.buffer);
            self.buffered = 0;
        }

        while input.len() >= 16 {
            consume_block(&mut self.lanes, &input[..16]);
            input = &input[16..];
        }

        self.buffer[..input.len()].copy_from_slice(input);
        self.buffered = input.len();
    }

    /// Returns the XXH32 digest without consuming the state.
    #[must_use]
    pub fn digest(&self) -> u32 {
        let mut hash = if self.length_overflowed || self.total_len >= 16 {
            self.lanes[0]
                .rotate_left(1)
                .wrapping_add(self.lanes[1].rotate_left(7))
                .wrapping_add(self.lanes[2].rotate_left(12))
                .wrapping_add(self.lanes[3].rotate_left(18))
        } else {
            self.seed.wrapping_add(PRIME5)
        };
        hash = hash.wrapping_add(self.total_len as u32);
        consume_tail(hash, &self.buffer[..self.buffered])
    }
}

#[inline(always)]
fn consume_block(lanes: &mut [u32; 4], block: &[u8]) {
    lanes[0] = round(lanes[0], read_u32(block, 0));
    lanes[1] = round(lanes[1], read_u32(block, 4));
    lanes[2] = round(lanes[2], read_u32(block, 8));
    lanes[3] = round(lanes[3], read_u32(block, 12));
}

impl Default for Xxh32 {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Xxh32 {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }

    #[inline]
    fn finish(&self) -> u64 {
        u64::from(self.digest())
    }
}

impl_std_io_write!(Xxh32);

/// Deterministic [`BuildHasher`] for [`Xxh32`].
///
/// XXH32 only provides 32 bits of output and is usually a poor choice for a
/// general-purpose `HashMap`; prefer [`crate::Xxh3Builder`] on 64-bit systems.
#[derive(Clone, Copy, Debug, Default)]
pub struct Xxh32Builder {
    seed: u32,
}

impl Xxh32Builder {
    /// Creates a builder using `seed`.
    #[must_use]
    pub const fn with_seed(seed: u32) -> Self {
        Self { seed }
    }
}

impl BuildHasher for Xxh32Builder {
    type Hasher = Xxh32;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Xxh32::with_seed(self.seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_overflow_keeps_long_digest_mode() {
        let mut hash = Xxh32::new();
        hash.total_len = u64::MAX;
        hash.update(&[0]);

        assert!(hash.length_overflowed);
        assert_eq!(hash.total_len(), u64::MAX);
        let expected = hash.lanes[0]
            .rotate_left(1)
            .wrapping_add(hash.lanes[1].rotate_left(7))
            .wrapping_add(hash.lanes[2].rotate_left(12))
            .wrapping_add(hash.lanes[3].rotate_left(18))
            .wrapping_add(hash.total_len as u32);
        assert_eq!(
            hash.digest(),
            consume_tail(expected, &hash.buffer[..hash.buffered])
        );
    }
}
