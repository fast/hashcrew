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

//! AArch64 CRC instructions and parallel polynomial folding.
//!
//! The folding schedules follow corsix/fast-crc32's `neon v12e_v1` and
//! `neon_eor3 v9s3x2e_s3`, also used by crc-fast. See LICENSE for the
//! upstream copyright notices and MIT terms.

use core::arch::aarch64::*;

use super::folding_factors;

#[inline]
pub(super) fn crc_available() -> bool {
    #[cfg(feature = "std")]
    {
        std::arch::is_aarch64_feature_detected!("crc")
    }
    #[cfg(not(feature = "std"))]
    {
        cfg!(target_feature = "crc")
    }
}

#[inline]
fn pmull_available() -> bool {
    #[cfg(feature = "std")]
    {
        std::arch::is_aarch64_feature_detected!("aes")
            && std::arch::is_aarch64_feature_detected!("pmull")
    }
    #[cfg(not(feature = "std"))]
    {
        cfg!(target_feature = "aes")
    }
}

#[inline]
fn sha3_available() -> bool {
    #[cfg(feature = "std")]
    {
        std::arch::is_aarch64_feature_detected!("sha3")
    }
    #[cfg(not(feature = "std"))]
    {
        cfg!(target_feature = "sha3")
    }
}

#[inline]
#[target_feature(enable = "crc")]
pub(super) unsafe fn update<const CASTAGNOLI: bool>(state: u32, input: &[u8]) -> u32 {
    // SAFETY: CRC is guaranteed by the caller. Polynomial folding additionally
    // requires AES/PMULL, checked before entering its target-feature function.
    unsafe {
        if input.len() >= 128 && pmull_available() {
            // Compile-time features let PMULL inline into the caller. Generic
            // targets reach the fused loop's crossover at a smaller input size.
            let crossover = if cfg!(all(
                target_feature = "crc",
                target_feature = "aes",
                target_feature = "sha3"
            )) {
                16_384
            } else {
                4096
            };
            if input.len() > crossover && sha3_available() {
                fusion::<CASTAGNOLI>(state, input)
            } else {
                pmull::<CASTAGNOLI>(state, input)
            }
        } else {
            native::<CASTAGNOLI>(state, input)
        }
    }
}

#[inline]
#[target_feature(enable = "crc")]
unsafe fn crc64<const CASTAGNOLI: bool>(state: u32, word: u64) -> u32 {
    if CASTAGNOLI {
        __crc32cd(state, word)
    } else {
        __crc32d(state, word)
    }
}

#[inline]
#[target_feature(enable = "crc")]
unsafe fn crc32<const CASTAGNOLI: bool>(state: u32, word: u32) -> u32 {
    if CASTAGNOLI {
        __crc32cw(state, word)
    } else {
        __crc32w(state, word)
    }
}

#[inline]
#[target_feature(enable = "crc")]
unsafe fn native<const CASTAGNOLI: bool>(mut state: u32, mut input: &[u8]) -> u32 {
    // SAFETY: CRC is guaranteed by the caller. Fixed-size chunks prove every
    // read is in bounds; from_le_bytes accepts all input alignments.
    unsafe {
        while let Some((words, tail)) = input.split_first_chunk::<64>() {
            macro_rules! step {
                ($offset:expr) => {
                    state = crc64::<CASTAGNOLI>(
                        state,
                        u64::from_le_bytes(words[$offset..$offset + 8].try_into().unwrap()),
                    );
                };
            }
            step!(0);
            step!(8);
            step!(16);
            step!(24);
            step!(32);
            step!(40);
            step!(48);
            step!(56);
            input = tail;
        }
        while let Some((word, tail)) = input.split_first_chunk::<8>() {
            state = crc64::<CASTAGNOLI>(state, u64::from_le_bytes(*word));
            input = tail;
        }
        if let Some((word, tail)) = input.split_first_chunk::<4>() {
            state = crc32::<CASTAGNOLI>(state, u32::from_le_bytes(*word));
            input = tail;
        }
        if let Some((word, tail)) = input.split_first_chunk::<2>() {
            let word = u16::from_le_bytes(*word);
            state = if CASTAGNOLI {
                __crc32ch(state, word)
            } else {
                __crc32h(state, word)
            };
            input = tail;
        }
        if let Some(&byte) = input.first() {
            state = if CASTAGNOLI {
                __crc32cb(state, byte)
            } else {
                __crc32b(state, byte)
            };
        }
        state
    }
}

#[inline]
#[target_feature(enable = "aes")]
unsafe fn fold(value: uint64x2_t, factors: uint64x2_t, next: uint64x2_t) -> uint64x2_t {
    let low = vmull_p64(vgetq_lane_u64(value, 0), vgetq_lane_u64(factors, 0));
    let high = vmull_high_p64(vreinterpretq_p64_u64(value), vreinterpretq_p64_u64(factors));
    veorq_u64(
        vreinterpretq_u64_p128(high),
        veorq_u64(vreinterpretq_u64_p128(low), next),
    )
}

#[inline]
#[target_feature(enable = "crc,aes")]
unsafe fn pmull<const CASTAGNOLI: bool>(mut state: u32, mut input: &[u8]) -> u32 {
    // SAFETY: The caller checks CRC and AES/PMULL. All vector loads belong to
    // checked 192- or 16-byte blocks, and AArch64 vector loads permit unaligned
    // pointers. No reference to an aligned u64 is formed from the byte slice.
    unsafe {
        if input.len() >= 192 {
            let mut ptr = input.as_ptr();
            let mut lanes = [vmovq_n_u64(0); 12];
            for (i, lane) in lanes.iter_mut().enumerate() {
                *lane = vld1q_u64(ptr.add(i * 16).cast());
            }
            lanes[0] = veorq_u64(lanes[0], vsetq_lane_u64(state as u64, vmovq_n_u64(0), 0));
            let factors = vld1q_u64(const { &folding_factors::<CASTAGNOLI>(192) }.as_ptr());
            let blocks = input.len() / 192;
            ptr = ptr.add(192);
            for _ in 1..blocks {
                macro_rules! step {
                    ($i:expr) => {
                        lanes[$i] = fold(lanes[$i], factors, vld1q_u64(ptr.add($i * 16).cast()));
                    };
                }
                step!(0);
                step!(1);
                step!(2);
                step!(3);
                step!(4);
                step!(5);
                step!(6);
                step!(7);
                step!(8);
                step!(9);
                step!(10);
                step!(11);
                ptr = ptr.add(192);
            }
            let factors = vld1q_u64(const { &folding_factors::<CASTAGNOLI>(16) }.as_ptr());
            lanes[0] = fold(lanes[0], factors, lanes[1]);
            lanes[2] = fold(lanes[2], factors, lanes[3]);
            lanes[4] = fold(lanes[4], factors, lanes[5]);
            lanes[6] = fold(lanes[6], factors, lanes[7]);
            lanes[8] = fold(lanes[8], factors, lanes[9]);
            lanes[10] = fold(lanes[10], factors, lanes[11]);
            let factors = vld1q_u64(const { &folding_factors::<CASTAGNOLI>(32) }.as_ptr());
            lanes[0] = fold(lanes[0], factors, lanes[2]);
            lanes[4] = fold(lanes[4], factors, lanes[6]);
            lanes[8] = fold(lanes[8], factors, lanes[10]);
            let factors = vld1q_u64(const { &folding_factors::<CASTAGNOLI>(64) }.as_ptr());
            lanes[0] = fold(lanes[0], factors, lanes[4]);
            lanes[0] = fold(lanes[0], factors, lanes[8]);
            state = crc64::<CASTAGNOLI>(0, vgetq_lane_u64(lanes[0], 0));
            state = crc64::<CASTAGNOLI>(state, vgetq_lane_u64(lanes[0], 1));
            input = &input[blocks * 192..];
        }
        if let Some((block, tail)) = input.split_first_chunk::<16>() {
            let mut lane = vld1q_u64(block.as_ptr().cast());
            lane = veorq_u64(lane, vsetq_lane_u64(state as u64, vmovq_n_u64(0), 0));
            let factors = vld1q_u64(const { &folding_factors::<CASTAGNOLI>(16) }.as_ptr());
            input = tail;
            while let Some((block, tail)) = input.split_first_chunk::<16>() {
                lane = fold(lane, factors, vld1q_u64(block.as_ptr().cast()));
                input = tail;
            }
            state = crc64::<CASTAGNOLI>(0, vgetq_lane_u64(lane, 0));
            state = crc64::<CASTAGNOLI>(state, vgetq_lane_u64(lane, 1));
        }
        native::<CASTAGNOLI>(state, input)
    }
}

#[inline]
#[target_feature(enable = "aes,sha3")]
unsafe fn fold3(value: uint64x2_t, factors: uint64x2_t, next: uint64x2_t) -> uint64x2_t {
    let low = vmull_p64(vgetq_lane_u64(value, 0), vgetq_lane_u64(factors, 0));
    let high = vmull_high_p64(vreinterpretq_p64_u64(value), vreinterpretq_p64_u64(factors));
    veor3q_u64(
        vreinterpretq_u64_p128(low),
        vreinterpretq_u64_p128(high),
        next,
    )
}

#[inline]
#[target_feature(enable = "crc,aes")]
unsafe fn shift<const CASTAGNOLI: bool>(state: u32, bytes: usize) -> uint64x2_t {
    // SAFETY: CRC and AES/PMULL are enabled. The caller shifts by at least
    // eight bytes. shift_start preserves the exponent for very large slices.
    unsafe {
        let (mut bits, mut stack) = super::shift_start(bytes);
        while bits > 191 {
            stack = (stack << 1) | (bits & 1);
            bits = (bits >> 1) - 16;
        }
        stack = !stack;
        let mut factor = 0x8000_0000 >> (bits & 31);
        for _ in 0..bits >> 5 {
            factor = crc32::<CASTAGNOLI>(factor, 0);
        }
        while stack > 1 {
            let low = (stack & 1) as u32;
            stack >>= 1;
            let value = vreinterpret_p8_u64(vmov_n_u64(factor as u64));
            let square = vgetq_lane_u64(vreinterpretq_u64_p16(vmull_p8(value, value)), 0);
            factor = crc64::<CASTAGNOLI>(0, square << low);
        }
        vreinterpretq_u64_p128(vmull_p64(state as u64, factor as u64))
    }
}

const fn tail_factors<const CASTAGNOLI: bool>() -> [[u64; 2]; 8] {
    let mut factors = [[0; 2]; 8];
    let mut groups = 0;
    while groups < factors.len() {
        factors[groups] = [
            super::power::<CASTAGNOLI>(groups * 128 + 31) as u64,
            super::power::<CASTAGNOLI>(groups * 64 + 31) as u64,
        ];
        groups += 1;
    }
    factors
}

#[inline]
#[target_feature(enable = "crc,aes")]
unsafe fn tail<const CASTAGNOLI: bool>(mut state: u32, mut input: &[u8]) -> u32 {
    // Three independent CRC streams avoid a long dependency chain after the
    // vector loop. Only seven short shifts are possible, so use constant factors.
    // SAFETY: CRC and AES/PMULL are enabled. Complete 24-byte groups plus the
    // final eight bytes bound every unaligned load and the factor table index.
    unsafe {
        if (32..192).contains(&input.len()) {
            let groups = (input.len() - 8) / 24;
            let stride = groups * 8;
            let ptr = input.as_ptr();
            let mut second = 0;
            let mut third = 0;
            for i in 0..groups {
                state = crc64::<CASTAGNOLI>(state, ptr.add(i * 8).cast::<u64>().read_unaligned());
                second = crc64::<CASTAGNOLI>(
                    second,
                    ptr.add(stride + i * 8).cast::<u64>().read_unaligned(),
                );
                third = crc64::<CASTAGNOLI>(
                    third,
                    ptr.add(stride * 2 + i * 8).cast::<u64>().read_unaligned(),
                );
            }
            let factors = const { tail_factors::<CASTAGNOLI>() }[groups];
            let a = vreinterpretq_u64_p128(vmull_p64(state as u64, factors[0]));
            let b = vreinterpretq_u64_p128(vmull_p64(second as u64, factors[1]));
            let last = ptr.add(groups * 24).cast::<u64>().read_unaligned();
            state = crc64::<CASTAGNOLI>(third, last ^ vgetq_lane_u64(veorq_u64(a, b), 0));
            input = &input[groups * 24 + 8..];
        }
        native::<CASTAGNOLI>(state, input)
    }
}

#[inline]
#[target_feature(enable = "crc,aes,sha3")]
unsafe fn fusion<const CASTAGNOLI: bool>(mut state: u32, input: &[u8]) -> u32 {
    // SAFETY: The caller enables CRC, AES/PMULL, and SHA3. Each complete
    // 192-byte group contributes 16 bytes to each of three scalar streams
    // and 144 bytes to the vector stream. Counted loops keep pointers inside
    // the allocation, including when there is only one group. Scalar loads
    // use read_unaligned; vector loads have no alignment requirement.
    unsafe {
        // Bulk vector loads that straddle cache lines are costly on some ARM
        // cores. A short prefix keeps all three scalar streams and vectors aligned.
        let prefix = input.as_ptr().align_offset(16).min(input.len());
        state = native::<CASTAGNOLI>(state, &input[..prefix]);
        let input = &input[prefix..];
        let blocks = input.len() / 192;
        if blocks == 0 {
            return native::<CASTAGNOLI>(state, input);
        }
        let scalar_len = blocks * 16;
        let mut ptr = input.as_ptr();
        let mut vector = ptr.add(scalar_len * 3);
        let mut lanes = [vmovq_n_u64(0); 9];
        for (i, lane) in lanes.iter_mut().enumerate() {
            *lane = vld1q_u64(vector.add(i * 16).cast());
        }
        vector = vector.add(144);
        let factors = vld1q_u64(const { &folding_factors::<CASTAGNOLI>(144) }.as_ptr());
        let mut second = 0;
        let mut third = 0;
        macro_rules! scalar_step {
            () => {
                state = crc64::<CASTAGNOLI>(state, ptr.cast::<u64>().read_unaligned());
                second =
                    crc64::<CASTAGNOLI>(second, ptr.add(scalar_len).cast::<u64>().read_unaligned());
                third = crc64::<CASTAGNOLI>(
                    third,
                    ptr.add(scalar_len * 2).cast::<u64>().read_unaligned(),
                );
                state = crc64::<CASTAGNOLI>(state, ptr.add(8).cast::<u64>().read_unaligned());
                second = crc64::<CASTAGNOLI>(
                    second,
                    ptr.add(scalar_len + 8).cast::<u64>().read_unaligned(),
                );
                third = crc64::<CASTAGNOLI>(
                    third,
                    ptr.add(scalar_len * 2 + 8).cast::<u64>().read_unaligned(),
                );
            };
        }
        for _ in 1..blocks {
            macro_rules! step {
                ($i:expr) => {
                    lanes[$i] = fold3(lanes[$i], factors, vld1q_u64(vector.add($i * 16).cast()));
                };
            }
            step!(0);
            step!(1);
            step!(2);
            step!(3);
            step!(4);
            step!(5);
            step!(6);
            step!(7);
            step!(8);
            scalar_step!();
            ptr = ptr.add(16);
            vector = vector.add(144);
        }
        scalar_step!();

        let factors = vld1q_u64(const { &folding_factors::<CASTAGNOLI>(16) }.as_ptr());
        lanes[0] = fold3(lanes[0], factors, lanes[1]);
        lanes[0] = fold3(lanes[0], factors, lanes[2]);
        lanes[3] = fold3(lanes[3], factors, lanes[4]);
        lanes[5] = fold3(lanes[5], factors, lanes[6]);
        lanes[7] = fold3(lanes[7], factors, lanes[8]);
        let factors = vld1q_u64(const { &folding_factors::<CASTAGNOLI>(32) }.as_ptr());
        lanes[0] = fold3(lanes[0], factors, lanes[3]);
        lanes[5] = fold3(lanes[5], factors, lanes[7]);
        let factors = vld1q_u64(const { &folding_factors::<CASTAGNOLI>(64) }.as_ptr());
        lanes[0] = fold3(lanes[0], factors, lanes[5]);

        let a = shift::<CASTAGNOLI>(state, scalar_len * 2 + blocks * 144);
        let b = shift::<CASTAGNOLI>(second, scalar_len + blocks * 144);
        let c = shift::<CASTAGNOLI>(third, blocks * 144);
        let shifted = vgetq_lane_u64(veor3q_u64(a, b, c), 0);
        state = crc64::<CASTAGNOLI>(0, vgetq_lane_u64(lanes[0], 0));
        state = crc64::<CASTAGNOLI>(state, vgetq_lane_u64(lanes[0], 1) ^ shifted);
        tail::<CASTAGNOLI>(state, &input[blocks * 192..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_backends_match_scalar() {
        if !std::arch::is_aarch64_feature_detected!("crc") {
            return;
        }
        // SAFETY: Each backend is checked for all of its required features.
        unsafe {
            super::super::test_backend::<false>(native::<false>);
            super::super::test_backend::<true>(native::<true>);
            if std::arch::is_aarch64_feature_detected!("aes")
                && std::arch::is_aarch64_feature_detected!("pmull")
            {
                super::super::test_backend::<false>(pmull::<false>);
                super::super::test_backend::<true>(pmull::<true>);
                super::super::test_backend::<false>(tail::<false>);
                super::super::test_backend::<true>(tail::<true>);
                super::super::test_shift::<false>(|state, bytes| {
                    crc64::<false>(0, vgetq_lane_u64(shift::<false>(state, bytes), 0))
                });
                super::super::test_shift::<true>(|state, bytes| {
                    crc64::<true>(0, vgetq_lane_u64(shift::<true>(state, bytes), 0))
                });
                if std::arch::is_aarch64_feature_detected!("sha3") {
                    super::super::test_backend::<false>(fusion::<false>);
                    super::super::test_backend::<true>(fusion::<true>);
                }
            }
        }
    }
}
