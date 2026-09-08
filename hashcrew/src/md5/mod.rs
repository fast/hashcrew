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

// Implements the MD5 algorithm specified in RFC 1321, section 3:
// https://www.rfc-editor.org/rfc/rfc1321.html#section-3

//! MD5 one-shot and streaming APIs for compatibility with existing digests.
//!
//! Call [`md5()`] for complete input or use [`Md5`] for incremental input.
//! Both return the standard 16 digest bytes in RFC 1321 order. Enable the `std`
//! feature to use [`Md5`] as
//! [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html).
//!
//! MD5 is cryptographically broken. Use it only for compatibility with existing
//! formats and protocols, not for security-sensitive applications.
//!
//! ```
//! use hashcrew::md5::Md5;
//! use hashcrew::md5::md5;
//!
//! let mut state = Md5::new();
//! state.update(b"hash");
//! state.update(b"crew");
//! assert_eq!(state.digest(), md5(b"hashcrew"));
//! ```
//!
//! # Hexadecimal output
//!
//! The returned bytes are already in standard MD5 order. To display the usual
//! 32-character hexadecimal checksum, preserve that order and any leading zeroes:
//!
//! ```
//! use hashcrew::md5::md5;
//!
//! let digest = md5(b"a");
//! let checksum = format!("{:032x}", u128::from_be_bytes(digest));
//! assert_eq!(checksum, "0cc175b9c0f1b6a831c399e269772661");
//! ```

use crate::read_u32;

const INITIAL_STATE: [u32; 4] = [0x6745_2301, 0xefcd_ab89, 0x98ba_dcfe, 0x1032_5476];

/// Hashes `input` with MD5, returning its standard 16-byte digest.
///
/// MD5 is provided for compatibility and is not cryptographically secure.
#[must_use]
#[inline]
pub fn md5(input: &[u8]) -> [u8; 16] {
    let mut state = Md5::new();
    state.update(input);
    state.digest()
}

/// Incremental MD5 state for compatibility with existing digests.
///
/// [`Self::digest`] preserves the state so it can be read repeatedly or updated
/// with more input. MD5 is not cryptographically secure.
#[derive(Clone, Debug)]
pub struct Md5 {
    state: [u32; 4],
    buffer: [u8; 64],
    buffered: usize,
    total_len: u64,
}

impl Md5 {
    /// Creates an MD5 state.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            state: INITIAL_STATE,
            buffer: [0; 64],
            buffered: 0,
            total_len: 0,
        }
    }

    /// Adds raw bytes to the hash state.
    #[inline]
    pub fn update(&mut self, mut input: &[u8]) {
        self.total_len = self.total_len.wrapping_add(input.len() as u64);

        if self.buffered != 0 {
            let copied = (64 - self.buffered).min(input.len());
            self.buffer[self.buffered..self.buffered + copied].copy_from_slice(&input[..copied]);
            self.buffered += copied;
            input = &input[copied..];
            if self.buffered < 64 {
                return;
            }
            compress(&mut self.state, &self.buffer);
            self.buffered = 0;
        }

        while let Some((block, rest)) = input.split_first_chunk::<64>() {
            compress(&mut self.state, block);
            input = rest;
        }

        self.buffer[..input.len()].copy_from_slice(input);
        self.buffered = input.len();
    }

    /// Returns the standard 16-byte MD5 digest of all input so far.
    /// Further updates extend the same message.
    #[must_use]
    #[inline]
    pub fn digest(&self) -> [u8; 16] {
        let mut state = self.state;
        let mut block = [0; 64];
        block[..self.buffered].copy_from_slice(&self.buffer[..self.buffered]);
        block[self.buffered] = 0x80;

        // Padding needs an extra block when the 64-bit length does not fit.
        if self.buffered >= 56 {
            compress(&mut state, &block);
            block = [0; 64];
        }
        block[56..].copy_from_slice(&self.total_len.wrapping_mul(8).to_le_bytes());
        compress(&mut state, &block);

        let mut digest = [0; 16];
        for (word, bytes) in state.iter().zip(digest.chunks_exact_mut(4)) {
            bytes.copy_from_slice(&word.to_le_bytes());
        }
        digest
    }
}

impl Default for Md5 {
    fn default() -> Self {
        Self::new()
    }
}

fn compress(state: &mut [u32; 4], block: &[u8; 64]) {
    let mut words = [0; 16];
    for (index, word) in words.iter_mut().enumerate() {
        *word = read_u32(block, index * 4);
    }
    let [mut a, mut b, mut c, mut d] = *state;

    // Keep the schedule explicit so rotations and word indexes are constants.
    macro_rules! step {
        ($a:ident, $b:ident, $mix:expr, $word:literal, $shift:literal, $constant:literal) => {
            $a = $b.wrapping_add(
                $a.wrapping_add($mix)
                    .wrapping_add(words[$word])
                    .wrapping_add($constant)
                    .rotate_left($shift),
            );
        };
    }

    // Round 1.
    step!(a, b, (b & c) | (!b & d), 0, 7, 0xd76a_a478);
    step!(d, a, (a & b) | (!a & c), 1, 12, 0xe8c7_b756);
    step!(c, d, (d & a) | (!d & b), 2, 17, 0x2420_70db);
    step!(b, c, (c & d) | (!c & a), 3, 22, 0xc1bd_ceee);
    step!(a, b, (b & c) | (!b & d), 4, 7, 0xf57c_0faf);
    step!(d, a, (a & b) | (!a & c), 5, 12, 0x4787_c62a);
    step!(c, d, (d & a) | (!d & b), 6, 17, 0xa830_4613);
    step!(b, c, (c & d) | (!c & a), 7, 22, 0xfd46_9501);
    step!(a, b, (b & c) | (!b & d), 8, 7, 0x6980_98d8);
    step!(d, a, (a & b) | (!a & c), 9, 12, 0x8b44_f7af);
    step!(c, d, (d & a) | (!d & b), 10, 17, 0xffff_5bb1);
    step!(b, c, (c & d) | (!c & a), 11, 22, 0x895c_d7be);
    step!(a, b, (b & c) | (!b & d), 12, 7, 0x6b90_1122);
    step!(d, a, (a & b) | (!a & c), 13, 12, 0xfd98_7193);
    step!(c, d, (d & a) | (!d & b), 14, 17, 0xa679_438e);
    step!(b, c, (c & d) | (!c & a), 15, 22, 0x49b4_0821);

    // Round 2.
    step!(a, b, (b & d) | (c & !d), 1, 5, 0xf61e_2562);
    step!(d, a, (a & c) | (b & !c), 6, 9, 0xc040_b340);
    step!(c, d, (d & b) | (a & !b), 11, 14, 0x265e_5a51);
    step!(b, c, (c & a) | (d & !a), 0, 20, 0xe9b6_c7aa);
    step!(a, b, (b & d) | (c & !d), 5, 5, 0xd62f_105d);
    step!(d, a, (a & c) | (b & !c), 10, 9, 0x0244_1453);
    step!(c, d, (d & b) | (a & !b), 15, 14, 0xd8a1_e681);
    step!(b, c, (c & a) | (d & !a), 4, 20, 0xe7d3_fbc8);
    step!(a, b, (b & d) | (c & !d), 9, 5, 0x21e1_cde6);
    step!(d, a, (a & c) | (b & !c), 14, 9, 0xc337_07d6);
    step!(c, d, (d & b) | (a & !b), 3, 14, 0xf4d5_0d87);
    step!(b, c, (c & a) | (d & !a), 8, 20, 0x455a_14ed);
    step!(a, b, (b & d) | (c & !d), 13, 5, 0xa9e3_e905);
    step!(d, a, (a & c) | (b & !c), 2, 9, 0xfcef_a3f8);
    step!(c, d, (d & b) | (a & !b), 7, 14, 0x676f_02d9);
    step!(b, c, (c & a) | (d & !a), 12, 20, 0x8d2a_4c8a);

    // Round 3.
    step!(a, b, b ^ c ^ d, 5, 4, 0xfffa_3942);
    step!(d, a, a ^ b ^ c, 8, 11, 0x8771_f681);
    step!(c, d, d ^ a ^ b, 11, 16, 0x6d9d_6122);
    step!(b, c, c ^ d ^ a, 14, 23, 0xfde5_380c);
    step!(a, b, b ^ c ^ d, 1, 4, 0xa4be_ea44);
    step!(d, a, a ^ b ^ c, 4, 11, 0x4bde_cfa9);
    step!(c, d, d ^ a ^ b, 7, 16, 0xf6bb_4b60);
    step!(b, c, c ^ d ^ a, 10, 23, 0xbebf_bc70);
    step!(a, b, b ^ c ^ d, 13, 4, 0x289b_7ec6);
    step!(d, a, a ^ b ^ c, 0, 11, 0xeaa1_27fa);
    step!(c, d, d ^ a ^ b, 3, 16, 0xd4ef_3085);
    step!(b, c, c ^ d ^ a, 6, 23, 0x0488_1d05);
    step!(a, b, b ^ c ^ d, 9, 4, 0xd9d4_d039);
    step!(d, a, a ^ b ^ c, 12, 11, 0xe6db_99e5);
    step!(c, d, d ^ a ^ b, 15, 16, 0x1fa2_7cf8);
    step!(b, c, c ^ d ^ a, 2, 23, 0xc4ac_5665);

    // Round 4.
    step!(a, b, c ^ (b | !d), 0, 6, 0xf429_2244);
    step!(d, a, b ^ (a | !c), 7, 10, 0x432a_ff97);
    step!(c, d, a ^ (d | !b), 14, 15, 0xab94_23a7);
    step!(b, c, d ^ (c | !a), 5, 21, 0xfc93_a039);
    step!(a, b, c ^ (b | !d), 12, 6, 0x655b_59c3);
    step!(d, a, b ^ (a | !c), 3, 10, 0x8f0c_cc92);
    step!(c, d, a ^ (d | !b), 10, 15, 0xffef_f47d);
    step!(b, c, d ^ (c | !a), 1, 21, 0x8584_5dd1);
    step!(a, b, c ^ (b | !d), 8, 6, 0x6fa8_7e4f);
    step!(d, a, b ^ (a | !c), 15, 10, 0xfe2c_e6e0);
    step!(c, d, a ^ (d | !b), 6, 15, 0xa301_4314);
    step!(b, c, d ^ (c | !a), 13, 21, 0x4e08_11a1);
    step!(a, b, c ^ (b | !d), 4, 6, 0xf753_7e82);
    step!(d, a, b ^ (a | !c), 11, 10, 0xbd3a_f235);
    step!(c, d, a ^ (d | !b), 2, 15, 0x2ad7_d2bb);
    step!(b, c, d ^ (c | !a), 9, 21, 0xeb86_d391);

    for (word, value) in state.iter_mut().zip([a, b, c, d]) {
        *word = word.wrapping_add(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc_vectors() {
        // RFC 1321, appendix A.5.
        let vectors: &[(&[u8], u128)] = &[
            (b"", 0xd41d8cd98f00b204e9800998ecf8427e),
            (b"a", 0x0cc175b9c0f1b6a831c399e269772661),
            (b"abc", 0x900150983cd24fb0d6963f7d28e17f72),
            (b"message digest", 0xf96b697d7cb7938d525a2f31aaf161d0),
            (
                b"abcdefghijklmnopqrstuvwxyz",
                0xc3fcd3d76192e4007dfb496cca67e13b,
            ),
            (
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
                0xd174ab98d277d9f5a5611c2c9f419d9f,
            ),
            (
                b"12345678901234567890123456789012345678901234567890123456789012345678901234567890",
                0x57edf4a22be3c955ac49da2e2107b67a,
            ),
        ];
        for &(input, expected) in vectors {
            assert_eq!(md5(input), expected.to_be_bytes());
            let mut state = Md5::new();
            for byte in input {
                state.update(core::slice::from_ref(byte));
            }
            assert_eq!(state.digest(), expected.to_be_bytes());
        }
    }

    #[test]
    fn digest_preserves_state_for_further_updates() {
        let mut state = Md5::default();
        state.update(b"a");
        assert_eq!(
            state.digest(),
            0x0cc175b9c0f1b6a831c399e269772661u128.to_be_bytes()
        );
        assert_eq!(
            state.digest(),
            0x0cc175b9c0f1b6a831c399e269772661u128.to_be_bytes()
        );

        state.update(b"bc");
        assert_eq!(
            state.digest(),
            0x900150983cd24fb0d6963f7d28e17f72u128.to_be_bytes()
        );
    }
}
