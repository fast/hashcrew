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

//! XXH64 one-shot and streaming APIs.

use core::hash::BuildHasher;
use core::hash::Hasher;

use crate::read_u32;
use crate::read_u64;

const PRIME1: u64 = 0x9e37_79b1_85eb_ca87;
const PRIME2: u64 = 0xc2b2_ae3d_27d4_eb4f;
const PRIME3: u64 = 0x1656_67b1_9e37_79f9;
const PRIME4: u64 = 0x85eb_ca77_c2b2_ae63;
const PRIME5: u64 = 0x27d4_eb2f_1656_67c5;

#[inline(always)]
fn round(acc: u64, lane: u64) -> u64 {
    acc.wrapping_add(lane.wrapping_mul(PRIME2))
        .rotate_left(31)
        .wrapping_mul(PRIME1)
}

#[inline(always)]
fn merge_round(acc: u64, lane: u64) -> u64 {
    (acc ^ round(0, lane))
        .wrapping_mul(PRIME1)
        .wrapping_add(PRIME4)
}

#[inline(always)]
fn avalanche(mut hash: u64) -> u64 {
    hash ^= hash >> 33;
    hash = hash.wrapping_mul(PRIME2);
    hash ^= hash >> 29;
    hash = hash.wrapping_mul(PRIME3);
    hash ^ (hash >> 32)
}

#[inline(always)]
fn consume_tail(mut hash: u64, tail: &[u8]) -> u64 {
    let mut offset = 0;
    while offset + 8 <= tail.len() {
        hash ^= round(0, read_u64(tail, offset));
        hash = hash
            .rotate_left(27)
            .wrapping_mul(PRIME1)
            .wrapping_add(PRIME4);
        offset += 8;
    }
    if offset + 4 <= tail.len() {
        hash ^= u64::from(read_u32(tail, offset)).wrapping_mul(PRIME1);
        hash = hash
            .rotate_left(23)
            .wrapping_mul(PRIME2)
            .wrapping_add(PRIME3);
        offset += 4;
    }
    for &byte in &tail[offset..] {
        hash ^= u64::from(byte).wrapping_mul(PRIME5);
        hash = hash.rotate_left(11).wrapping_mul(PRIME1);
    }
    avalanche(hash)
}

/// Hashes `input` with XXH64 and `seed`.
#[must_use]
#[inline]
pub fn xxh64(input: &[u8], seed: u64) -> u64 {
    let mut offset = 0;
    let mut lanes = [
        seed.wrapping_add(PRIME1).wrapping_add(PRIME2),
        seed.wrapping_add(PRIME2),
        seed,
        seed.wrapping_sub(PRIME1),
    ];

    while offset + 32 <= input.len() {
        lanes[0] = round(lanes[0], read_u64(input, offset));
        lanes[1] = round(lanes[1], read_u64(input, offset + 8));
        lanes[2] = round(lanes[2], read_u64(input, offset + 16));
        lanes[3] = round(lanes[3], read_u64(input, offset + 24));
        offset += 32;
    }

    let mut hash = if input.len() >= 32 {
        let hash = lanes[0]
            .rotate_left(1)
            .wrapping_add(lanes[1].rotate_left(7))
            .wrapping_add(lanes[2].rotate_left(12))
            .wrapping_add(lanes[3].rotate_left(18));
        let hash = merge_round(hash, lanes[0]);
        let hash = merge_round(hash, lanes[1]);
        let hash = merge_round(hash, lanes[2]);
        merge_round(hash, lanes[3])
    } else {
        seed.wrapping_add(PRIME5)
    };
    hash = hash.wrapping_add(input.len() as u64);
    consume_tail(hash, &input[offset..])
}

/// Incremental XXH64 state.
#[derive(Clone, Debug)]
pub struct Xxh64 {
    seed: u64,
    lanes: [u64; 4],
    buffer: [u8; 32],
    buffered: usize,
    total_len: u64,
    length_overflowed: bool,
}

impl Xxh64 {
    /// Creates an unseeded XXH64 state.
    #[must_use]
    pub const fn new() -> Self {
        Self::with_seed(0)
    }

    /// Hashes a complete byte slice without constructing streaming state.
    #[must_use]
    #[inline]
    pub fn oneshot(input: &[u8], seed: u64) -> u64 {
        xxh64(input, seed)
    }

    /// Creates an XXH64 state with `seed`.
    #[must_use]
    pub const fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            lanes: [
                seed.wrapping_add(PRIME1).wrapping_add(PRIME2),
                seed.wrapping_add(PRIME2),
                seed,
                seed.wrapping_sub(PRIME1),
            ],
            buffer: [0; 32],
            buffered: 0,
            total_len: 0,
            length_overflowed: false,
        }
    }

    /// Returns the seed used by this state.
    #[must_use]
    pub const fn seed(&self) -> u64 {
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
            let needed = 32 - self.buffered;
            let copied = needed.min(input.len());
            self.buffer[self.buffered..self.buffered + copied].copy_from_slice(&input[..copied]);
            self.buffered += copied;
            input = &input[copied..];
            if self.buffered < 32 {
                return;
            }
            consume_block(&mut self.lanes, &self.buffer);
            self.buffered = 0;
        }

        while input.len() >= 32 {
            consume_block(&mut self.lanes, &input[..32]);
            input = &input[32..];
        }

        self.buffer[..input.len()].copy_from_slice(input);
        self.buffered = input.len();
    }

    /// Returns the XXH64 digest without consuming the state.
    #[must_use]
    pub fn digest(&self) -> u64 {
        let mut hash = if self.length_overflowed || self.total_len >= 32 {
            let hash = self.lanes[0]
                .rotate_left(1)
                .wrapping_add(self.lanes[1].rotate_left(7))
                .wrapping_add(self.lanes[2].rotate_left(12))
                .wrapping_add(self.lanes[3].rotate_left(18));
            let hash = merge_round(hash, self.lanes[0]);
            let hash = merge_round(hash, self.lanes[1]);
            let hash = merge_round(hash, self.lanes[2]);
            merge_round(hash, self.lanes[3])
        } else {
            self.seed.wrapping_add(PRIME5)
        };
        hash = hash.wrapping_add(self.total_len);
        consume_tail(hash, &self.buffer[..self.buffered])
    }
}

#[inline(always)]
fn consume_block(lanes: &mut [u64; 4], block: &[u8]) {
    lanes[0] = round(lanes[0], read_u64(block, 0));
    lanes[1] = round(lanes[1], read_u64(block, 8));
    lanes[2] = round(lanes[2], read_u64(block, 16));
    lanes[3] = round(lanes[3], read_u64(block, 24));
}

impl Default for Xxh64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Xxh64 {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.digest()
    }
}

#[cfg(feature = "std")]
impl std::io::Write for Xxh64 {
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

/// Deterministic [`BuildHasher`] for [`Xxh64`].
///
/// This builder is intended for trusted inputs. It does not randomize its seed
/// and is not resistant to deliberate hash-flooding attacks.
#[derive(Clone, Copy, Debug, Default)]
pub struct Xxh64Builder {
    seed: u64,
}

impl Xxh64Builder {
    /// Creates a builder using `seed`.
    #[must_use]
    pub const fn with_seed(seed: u64) -> Self {
        Self { seed }
    }
}

impl BuildHasher for Xxh64Builder {
    type Hasher = Xxh64;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Xxh64::with_seed(self.seed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn length_overflow_keeps_long_digest_mode() {
        let mut hash = Xxh64::new();
        hash.total_len = u64::MAX;
        hash.update(&[0]);

        assert!(hash.length_overflowed);
        assert_eq!(hash.total_len(), u64::MAX);
        let expected = hash.lanes[0]
            .rotate_left(1)
            .wrapping_add(hash.lanes[1].rotate_left(7))
            .wrapping_add(hash.lanes[2].rotate_left(12))
            .wrapping_add(hash.lanes[3].rotate_left(18));
        let expected = merge_round(expected, hash.lanes[0]);
        let expected = merge_round(expected, hash.lanes[1]);
        let expected = merge_round(expected, hash.lanes[2]);
        let expected = merge_round(expected, hash.lanes[3]).wrapping_add(hash.total_len);
        assert_eq!(
            hash.digest(),
            consume_tail(expected, &hash.buffer[..hash.buffered])
        );
    }
}
