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

//! x86-64 CRC32C instructions and polynomial folding for both variants.
//!
//! Folding schedules follow corsix/fast-crc32 and crc-fast. See the packaged
//! THIRD-PARTY-NOTICES for attribution.

// These intrinsics are unsafe on the MSRV but safe on newer compilers.
#![allow(unused_unsafe)]

use core::arch::x86_64::*;

use super::folding_factors;

#[inline]
fn crc_available() -> bool {
    #[cfg(feature = "std")]
    {
        std::arch::is_x86_feature_detected!("sse4.2")
    }
    #[cfg(not(feature = "std"))]
    {
        cfg!(target_feature = "sse4.2")
    }
}

#[inline]
fn pclmul_available() -> bool {
    #[cfg(feature = "std")]
    {
        std::arch::is_x86_feature_detected!("pclmulqdq")
    }
    #[cfg(not(feature = "std"))]
    {
        cfg!(target_feature = "pclmulqdq")
    }
}

#[cfg(crc_vpclmulqdq)]
#[inline]
fn avx2_available() -> bool {
    #[cfg(feature = "std")]
    {
        std::arch::is_x86_feature_detected!("avx2")
            && std::arch::is_x86_feature_detected!("vpclmulqdq")
    }
    #[cfg(not(feature = "std"))]
    {
        cfg!(target_feature = "avx2") && cfg!(target_feature = "vpclmulqdq")
    }
}

#[cfg(crc_vpclmulqdq)]
#[inline]
fn wide_available() -> bool {
    #[cfg(feature = "std")]
    {
        std::arch::is_x86_feature_detected!("avx512f")
            && std::arch::is_x86_feature_detected!("vpclmulqdq")
    }
    #[cfg(not(feature = "std"))]
    {
        cfg!(target_feature = "avx512f") && cfg!(target_feature = "vpclmulqdq")
    }
}

#[inline]
pub(super) fn update<const CASTAGNOLI: bool>(state: u32, input: &[u8]) -> u32 {
    if CASTAGNOLI && crc_available() && input.len() <= 256 {
        // SAFETY: SSE4.2 is available; native only reads inside the slice.
        return unsafe { native(state, input) };
    }
    if input.len() >= 16 && crc_available() && pclmul_available() {
        #[cfg(crc_vpclmulqdq)]
        if input.len() >= 384 && wide_available() {
            // SAFETY: All features used by wide were checked, including OS
            // support for saving the extended vector register state.
            return unsafe { wide::<CASTAGNOLI>(state, input) };
        }
        #[cfg(crc_vpclmulqdq)]
        if input.len() >= 256 && avx2_available() {
            // SAFETY: AVX2 and VPCLMULQDQ, including OS support, were checked.
            return unsafe { avx2::<CASTAGNOLI>(state, input) };
        }
        if CASTAGNOLI {
            // SAFETY: SSE4.2 and PCLMULQDQ were checked above.
            return unsafe { fusion(state, input) };
        }
        // SAFETY: SSE4.2 and PCLMULQDQ were checked above.
        return unsafe { pclmul::<CASTAGNOLI>(state, input) };
    }
    if CASTAGNOLI && crc_available() {
        // SAFETY: SSE4.2 is available even when polynomial multiplication is not.
        return unsafe { native(state, input) };
    }
    super::scalar::<CASTAGNOLI>(state, input)
}

#[inline]
#[target_feature(enable = "sse4.2")]
unsafe fn native(mut state: u32, mut input: &[u8]) -> u32 {
    // SAFETY: The caller enables SSE4.2. Fixed-size chunks prove all reads
    // are in bounds, and from_le_bytes accepts unaligned inputs.
    unsafe {
        while let Some((block, tail)) = input.split_first_chunk::<64>() {
            macro_rules! step {
                ($offset:expr) => {
                    state = _mm_crc32_u64(
                        state as u64,
                        u64::from_le_bytes(block[$offset..$offset + 8].try_into().unwrap()),
                    ) as u32;
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
            state = _mm_crc32_u64(state as u64, u64::from_le_bytes(*word)) as u32;
            input = tail;
        }
        if let Some((word, tail)) = input.split_first_chunk::<4>() {
            state = _mm_crc32_u32(state, u32::from_le_bytes(*word));
            input = tail;
        }
        if let Some((word, tail)) = input.split_first_chunk::<2>() {
            state = _mm_crc32_u16(state, u16::from_le_bytes(*word));
            input = tail;
        }
        if let Some(&byte) = input.first() {
            state = _mm_crc32_u8(state, byte);
        }
        state
    }
}

#[inline]
#[target_feature(enable = "pclmulqdq")]
unsafe fn fold(value: __m128i, factors: __m128i, next: __m128i) -> __m128i {
    // SAFETY: PCLMULQDQ is enabled; these operations only use registers.
    unsafe {
        let low = _mm_clmulepi64_si128(value, factors, 0);
        let high = _mm_clmulepi64_si128(value, factors, 17);
        _mm_xor_si128(_mm_xor_si128(low, high), next)
    }
}

#[inline]
#[target_feature(enable = "sse4.2")]
unsafe fn reduce<const CASTAGNOLI: bool>(value: __m128i) -> u32 {
    // SAFETY: SSE4.2 is enabled. The store addresses all 16 bytes of the
    // local array and has no alignment requirement.
    unsafe {
        if CASTAGNOLI {
            let state = _mm_crc32_u64(0, _mm_cvtsi128_si64(value) as u64);
            _mm_crc32_u64(state, _mm_cvtsi128_si64(_mm_srli_si128(value, 8)) as u64) as u32
        } else {
            let mut bytes = [0; 16];
            _mm_storeu_si128(bytes.as_mut_ptr().cast(), value);
            super::scalar::<false>(0, &bytes)
        }
    }
}

#[inline]
#[target_feature(enable = "sse4.2,pclmulqdq")]
unsafe fn pclmul<const CASTAGNOLI: bool>(mut state: u32, mut input: &[u8]) -> u32 {
    // SAFETY: SSE4.2 and PCLMULQDQ are enabled. Vector loads are limited to
    // complete 64- or 16-byte blocks and accept unaligned pointers.
    unsafe {
        let full = input;
        let factors = _mm_loadu_si128(const { &folding_factors::<CASTAGNOLI>(16) }.as_ptr().cast());
        let mut value;
        if input.len() >= 64 {
            let ptr = input.as_ptr();
            let mut a = _mm_xor_si128(_mm_loadu_si128(ptr.cast()), _mm_cvtsi32_si128(state as i32));
            let mut b = _mm_loadu_si128(ptr.add(16).cast());
            let mut c = _mm_loadu_si128(ptr.add(32).cast());
            let mut d = _mm_loadu_si128(ptr.add(48).cast());
            let stride =
                _mm_loadu_si128(const { &folding_factors::<CASTAGNOLI>(64) }.as_ptr().cast());
            input = &input[64..];
            while let Some((block, tail)) = input.split_first_chunk::<64>() {
                let ptr = block.as_ptr();
                a = fold(a, stride, _mm_loadu_si128(ptr.cast()));
                b = fold(b, stride, _mm_loadu_si128(ptr.add(16).cast()));
                c = fold(c, stride, _mm_loadu_si128(ptr.add(32).cast()));
                d = fold(d, stride, _mm_loadu_si128(ptr.add(48).cast()));
                input = tail;
            }
            value = fold(fold(fold(a, factors, b), factors, c), factors, d);
        } else if let Some((block, tail)) = input.split_first_chunk::<16>() {
            value = _mm_xor_si128(
                _mm_loadu_si128(block.as_ptr().cast()),
                _mm_cvtsi32_si128(state as i32),
            );
            input = tail;
        } else {
            return if CASTAGNOLI {
                native(state, input)
            } else {
                super::scalar::<false>(state, input)
            };
        }
        while let Some((block, tail)) = input.split_first_chunk::<16>() {
            value = fold(value, factors, _mm_loadu_si128(block.as_ptr().cast()));
            input = tail;
        }
        if !CASTAGNOLI && !input.is_empty() {
            const SHUFFLE: [u8; 32] = [
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0x80, 0x81, 0x82, 0x83, 0x84,
                0x85, 0x86, 0x87, 0x88, 0x89, 0x8a, 0x8b, 0x8c, 0x8d, 0x8e, 0x8f,
            ];
            // Move the low n bytes to the end of one vector, then join the
            // remaining bytes with the new tail in another. Leading zeroes
            // do not affect a zero-initialized CRC, so one fold consumes n bytes.
            // full has at least 16 bytes here; this overlapping load ends
            // exactly at the slice boundary, and n is in 1..16.
            let right = _mm_loadu_si128(SHUFFLE.as_ptr().add(input.len()).cast());
            let left = _mm_xor_si128(right, _mm_set1_epi8(i8::MIN));
            let next = _mm_loadu_si128(full.as_ptr().add(full.len() - 16).cast());
            let next = _mm_blendv_epi8(next, _mm_shuffle_epi8(value, right), left);
            value = fold(_mm_shuffle_epi8(value, left), factors, next);
            return reduce::<false>(value);
        }
        state = reduce::<CASTAGNOLI>(value);
        if CASTAGNOLI {
            native(state, input)
        } else {
            super::scalar::<false>(state, input)
        }
    }
}

#[inline]
#[target_feature(enable = "sse4.2,pclmulqdq")]
unsafe fn shift(state: u32, bytes: usize) -> u64 {
    // SAFETY: The caller enables SSE4.2 and PCLMULQDQ and shifts by at least
    // eight bytes. Only register operations are used.
    unsafe {
        let (mut bits, mut stack) = super::shift_start(bytes);
        while bits > 191 {
            stack = (stack << 1) | (bits & 1);
            bits = (bits >> 1) - 16;
        }
        stack = !stack;
        let mut factor = 0x8000_0000 >> (bits & 31);
        for _ in 0..bits >> 5 {
            factor = _mm_crc32_u32(factor, 0);
        }
        while stack > 1 {
            let low = stack & 1;
            stack >>= 1;
            let value = _mm_cvtsi32_si128(factor as i32);
            let square = _mm_cvtsi128_si64(_mm_clmulepi64_si128(value, value, 0)) as u64;
            factor = _mm_crc32_u64(0, square << low) as u32;
        }
        _mm_cvtsi128_si64(_mm_clmulepi64_si128(
            _mm_cvtsi32_si128(state as i32),
            _mm_cvtsi32_si128(factor as i32),
            0,
        )) as u64
    }
}

#[inline]
#[target_feature(enable = "sse4.2,pclmulqdq")]
unsafe fn fusion(state: u32, input: &[u8]) -> u32 {
    // Four vector streams run alongside three native CRC streams, following
    // corsix's v4s3x3 schedule. This also serves CPUs without wide VPCLMULQDQ.
    // SAFETY: SSE4.2 and PCLMULQDQ are enabled. Each group contains 64 vector
    // bytes and 3 * 24 scalar bytes, with eight more bytes reserved for merging.
    // Every pointer stays in the slice; all loads accept unaligned addresses.
    unsafe {
        if input.len() < 144 {
            return native(state, input);
        }
        let groups = (input.len() - 8) / 136;
        let scalar_len = groups * 24;
        let mut vector = input.as_ptr();
        let mut ptr = vector.add(groups * 64);
        let mut lanes = [_mm_setzero_si128(); 4];
        for (i, lane) in lanes.iter_mut().enumerate() {
            *lane = _mm_loadu_si128(vector.add(i * 16).cast());
        }
        lanes[0] = _mm_xor_si128(lanes[0], _mm_cvtsi32_si128(state as i32));
        vector = vector.add(64);
        let mut first = 0_u64;
        let mut second = 0_u64;
        let mut third = 0_u64;
        let factors = _mm_loadu_si128(const { &folding_factors::<true>(64) }.as_ptr().cast());
        macro_rules! scalar_step {
            ($offset:expr) => {
                first = _mm_crc32_u64(first, ptr.add($offset).cast::<u64>().read_unaligned());
                second = _mm_crc32_u64(
                    second,
                    ptr.add(scalar_len + $offset).cast::<u64>().read_unaligned(),
                );
                third = _mm_crc32_u64(
                    third,
                    ptr.add(scalar_len * 2 + $offset)
                        .cast::<u64>()
                        .read_unaligned(),
                );
            };
        }
        for _ in 1..groups {
            macro_rules! step {
                ($i:expr) => {
                    lanes[$i] = fold(
                        lanes[$i],
                        factors,
                        _mm_loadu_si128(vector.add($i * 16).cast()),
                    );
                };
            }
            step!(0);
            step!(1);
            step!(2);
            step!(3);
            scalar_step!(0);
            scalar_step!(8);
            scalar_step!(16);
            vector = vector.add(64);
            ptr = ptr.add(24);
        }
        scalar_step!(0);
        scalar_step!(8);
        scalar_step!(16);
        let factors = _mm_loadu_si128(const { &folding_factors::<true>(16) }.as_ptr().cast());
        lanes[0] = fold(lanes[0], factors, lanes[1]);
        lanes[2] = fold(lanes[2], factors, lanes[3]);
        let factors = _mm_loadu_si128(const { &folding_factors::<true>(32) }.as_ptr().cast());
        lanes[0] = fold(lanes[0], factors, lanes[2]);
        let merged = shift(first as u32, scalar_len * 2 + 8)
            ^ shift(second as u32, scalar_len + 8)
            ^ shift(reduce::<true>(lanes[0]), scalar_len * 3 + 8);
        let last = input
            .as_ptr()
            .add(groups * 136)
            .cast::<u64>()
            .read_unaligned();
        let state = _mm_crc32_u64(third, last ^ merged) as u32;
        native(state, &input[groups * 136 + 8..])
    }
}

#[cfg(crc_vpclmulqdq)]
#[allow(clippy::incompatible_msrv)] // Compiled only on Rust 1.89+ by build.rs.
#[inline]
#[target_feature(enable = "sse4.2,pclmulqdq,avx2,vpclmulqdq")]
unsafe fn avx2<const CASTAGNOLI: bool>(state: u32, input: &[u8]) -> u32 {
    // Eight 128-bit lanes keep the carryless multipliers busy without AVX-512.
    // SAFETY: The caller checks every enabled feature. Each unaligned vector
    // load lies in a complete 128-byte block, with the remainder handled below.
    unsafe {
        if input.len() < 128 {
            return pclmul::<CASTAGNOLI>(state, input);
        }
        let blocks = input.len() / 128;
        let mut ptr = input.as_ptr();
        let mut a = _mm256_loadu_si256(ptr.cast());
        let mut b = _mm256_loadu_si256(ptr.add(32).cast());
        let mut c = _mm256_loadu_si256(ptr.add(64).cast());
        let mut d = _mm256_loadu_si256(ptr.add(96).cast());
        a = _mm256_xor_si256(a, _mm256_setr_epi64x(state as i64, 0, 0, 0));
        ptr = ptr.add(128);
        let factors = _mm256_broadcastsi128_si256(_mm_loadu_si128(
            const { &folding_factors::<CASTAGNOLI>(128) }
                .as_ptr()
                .cast(),
        ));
        macro_rules! step {
            ($value:ident, $offset:expr) => {
                let low = _mm256_clmulepi64_epi128($value, factors, 0);
                let high = _mm256_clmulepi64_epi128($value, factors, 17);
                $value = _mm256_xor_si256(
                    _mm256_xor_si256(low, high),
                    _mm256_loadu_si256(ptr.add($offset).cast()),
                );
            };
        }
        for _ in 1..blocks {
            step!(a, 0);
            step!(b, 32);
            step!(c, 64);
            step!(d, 96);
            ptr = ptr.add(128);
        }
        let factors = _mm256_broadcastsi128_si256(_mm_loadu_si128(
            const { &folding_factors::<CASTAGNOLI>(32) }.as_ptr().cast(),
        ));
        a = _mm256_xor_si256(
            _mm256_xor_si256(
                _mm256_clmulepi64_epi128(a, factors, 0),
                _mm256_clmulepi64_epi128(a, factors, 17),
            ),
            b,
        );
        c = _mm256_xor_si256(
            _mm256_xor_si256(
                _mm256_clmulepi64_epi128(c, factors, 0),
                _mm256_clmulepi64_epi128(c, factors, 17),
            ),
            d,
        );
        let factors = _mm256_broadcastsi128_si256(_mm_loadu_si128(
            const { &folding_factors::<CASTAGNOLI>(64) }.as_ptr().cast(),
        ));
        a = _mm256_xor_si256(
            _mm256_xor_si256(
                _mm256_clmulepi64_epi128(a, factors, 0),
                _mm256_clmulepi64_epi128(a, factors, 17),
            ),
            c,
        );
        let factors = _mm_loadu_si128(const { &folding_factors::<CASTAGNOLI>(16) }.as_ptr().cast());
        let value = fold(
            _mm256_castsi256_si128(a),
            factors,
            _mm256_extracti128_si256(a, 1),
        );
        let state = reduce::<CASTAGNOLI>(value);
        pclmul::<CASTAGNOLI>(state, &input[blocks * 128..])
    }
}

#[cfg(crc_vpclmulqdq)]
#[allow(clippy::incompatible_msrv)] // Compiled only on Rust 1.89+ by build.rs.
#[inline]
#[target_feature(enable = "sse4.2,pclmulqdq,avx512f,vpclmulqdq")]
unsafe fn wide<const CASTAGNOLI: bool>(state: u32, input: &[u8]) -> u32 {
    // SAFETY: The caller checks every enabled feature. Each vector load lies
    // in a complete 192-byte group. All loads permit unaligned pointers.
    unsafe {
        let blocks = input.len() / 192;
        if blocks == 0 {
            return pclmul::<CASTAGNOLI>(state, input);
        }
        let mut ptr = input.as_ptr();
        let mut a = _mm512_loadu_si512(ptr.cast());
        let mut b = _mm512_loadu_si512(ptr.add(64).cast());
        let mut c = _mm512_loadu_si512(ptr.add(128).cast());
        // Zero every upper lane; a cast from __m128i alone leaves them unspecified.
        let initial =
            _mm512_maskz_mov_epi32(1, _mm512_castsi128_si512(_mm_cvtsi32_si128(state as i32)));
        a = _mm512_xor_si512(a, initial);
        ptr = ptr.add(192);
        let factors = _mm512_broadcast_i32x4(_mm_loadu_si128(
            const { &folding_factors::<CASTAGNOLI>(192) }
                .as_ptr()
                .cast(),
        ));
        macro_rules! step {
            ($value:ident, $offset:expr) => {
                let low = _mm512_clmulepi64_epi128($value, factors, 0);
                let high = _mm512_clmulepi64_epi128($value, factors, 17);
                $value = _mm512_ternarylogic_epi64(
                    low,
                    high,
                    _mm512_loadu_si512(ptr.add($offset).cast()),
                    0x96,
                );
            };
        }
        for _ in 1..blocks {
            step!(a, 0);
            step!(b, 64);
            step!(c, 128);
            ptr = ptr.add(192);
        }
        let factors = _mm512_broadcast_i32x4(_mm_loadu_si128(
            const { &folding_factors::<CASTAGNOLI>(64) }.as_ptr().cast(),
        ));
        a = _mm512_ternarylogic_epi64(
            _mm512_clmulepi64_epi128(a, factors, 0),
            _mm512_clmulepi64_epi128(a, factors, 17),
            b,
            0x96,
        );
        a = _mm512_ternarylogic_epi64(
            _mm512_clmulepi64_epi128(a, factors, 0),
            _mm512_clmulepi64_epi128(a, factors, 17),
            c,
            0x96,
        );
        let mut value = _mm512_castsi512_si128(a);
        let factors = _mm_loadu_si128(const { &folding_factors::<CASTAGNOLI>(16) }.as_ptr().cast());
        value = fold(value, factors, _mm512_extracti32x4_epi32(a, 1));
        value = fold(value, factors, _mm512_extracti32x4_epi32(a, 2));
        value = fold(value, factors, _mm512_extracti32x4_epi32(a, 3));
        let state = reduce::<CASTAGNOLI>(value);
        pclmul::<CASTAGNOLI>(state, &input[blocks * 192..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn available_backends_match_scalar() {
        if !std::arch::is_x86_feature_detected!("sse4.2") {
            return;
        }
        // SAFETY: Each backend is checked for all of its required features.
        unsafe {
            super::super::test_backend::<true>(native);
            if std::arch::is_x86_feature_detected!("pclmulqdq") {
                super::super::test_backend::<false>(pclmul::<false>);
                super::super::test_backend::<true>(pclmul::<true>);
                super::super::test_backend::<true>(fusion);
                super::super::test_shift::<true>(|state, bytes| {
                    _mm_crc32_u64(0, shift(state, bytes)) as u32
                });
                #[cfg(crc_vpclmulqdq)]
                if avx2_available() {
                    super::super::test_backend::<false>(avx2::<false>);
                    super::super::test_backend::<true>(avx2::<true>);
                }
                #[cfg(crc_vpclmulqdq)]
                if std::arch::is_x86_feature_detected!("avx512f")
                    && std::arch::is_x86_feature_detected!("vpclmulqdq")
                {
                    super::super::test_backend::<false>(wide::<false>);
                    super::super::test_backend::<true>(wide::<true>);
                }
            }
        }
    }
}
