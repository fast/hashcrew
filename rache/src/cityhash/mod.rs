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

//! CityHash 1.1.1 one-shot APIs.
//!
//! CityHash depends on the complete input length and tail, so this module does
//! not expose a streaming state that would need to retain the entire message.

use crate::fmix32;
use crate::read_u32;
use crate::read_u64;

const K0: u64 = 0xc3a5_c85c_97cb_3127;
const K1: u64 = 0xb492_b66f_be98_f273;
const K2: u64 = 0x9ae1_6a3b_2f90_404f;
const C1: u32 = 0xcc9e_2d51;
const C2: u32 = 0x1b87_3593;
const HASH128_MUL: u64 = 0x9ddf_ea08_eb38_2d69;

#[inline(always)]
fn mur32(mut value: u32, mut hash: u32) -> u32 {
    value = value.wrapping_mul(C1).rotate_right(17).wrapping_mul(C2);
    hash ^= value;
    hash.rotate_right(19)
        .wrapping_mul(5)
        .wrapping_add(0xe654_6b64)
}

#[inline]
fn hash32_len_0_to_4(input: &[u8]) -> u32 {
    let mut b = 0_u32;
    let mut c = 9_u32;
    for &byte in input {
        b = b.wrapping_mul(C1).wrapping_add((byte as i8) as u32);
        c ^= b;
    }
    fmix32(mur32(b, mur32(input.len() as u32, c)))
}

#[inline]
fn hash32_len_5_to_12(input: &[u8]) -> u32 {
    let len = input.len();
    let mut a = len as u32;
    let mut b = a.wrapping_mul(5);
    let mut c = 9_u32;
    let d = b;
    a = a.wrapping_add(read_u32(input, 0));
    b = b.wrapping_add(read_u32(input, len - 4));
    c = c.wrapping_add(read_u32(input, (len >> 1) & 4));
    fmix32(mur32(c, mur32(b, mur32(a, d))))
}

#[inline]
fn hash32_len_13_to_24(input: &[u8]) -> u32 {
    let len = input.len();
    let a = read_u32(input, (len >> 1) - 4);
    let b = read_u32(input, 4);
    let c = read_u32(input, len - 8);
    let d = read_u32(input, len >> 1);
    let e = read_u32(input, 0);
    let f = read_u32(input, len - 4);
    fmix32(mur32(
        f,
        mur32(e, mur32(d, mur32(c, mur32(b, mur32(a, len as u32))))),
    ))
}

/// Hashes `input` with CityHash32 1.1.1.
#[must_use]
#[inline]
pub fn cityhash32(input: &[u8]) -> u32 {
    let len = input.len();
    if len <= 24 {
        return if len <= 12 {
            if len <= 4 {
                hash32_len_0_to_4(input)
            } else {
                hash32_len_5_to_12(input)
            }
        } else {
            hash32_len_13_to_24(input)
        };
    }

    let len32 = len as u32;
    let mut hash = len32;
    let mut g = C1.wrapping_mul(len32);
    let mut f = g;
    let a0 = read_u32(input, len - 4)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let a1 = read_u32(input, len - 8)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let a2 = read_u32(input, len - 16)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let a3 = read_u32(input, len - 12)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);
    let a4 = read_u32(input, len - 20)
        .wrapping_mul(C1)
        .rotate_right(17)
        .wrapping_mul(C2);

    hash ^= a0;
    hash = hash
        .rotate_right(19)
        .wrapping_mul(5)
        .wrapping_add(0xe654_6b64);
    hash ^= a2;
    hash = hash
        .rotate_right(19)
        .wrapping_mul(5)
        .wrapping_add(0xe654_6b64);
    g ^= a1;
    g = g.rotate_right(19).wrapping_mul(5).wrapping_add(0xe654_6b64);
    g ^= a3;
    g = g.rotate_right(19).wrapping_mul(5).wrapping_add(0xe654_6b64);
    f = f.wrapping_add(a4);
    f = f.rotate_right(19).wrapping_mul(5).wrapping_add(0xe654_6b64);

    let body_len = ((len - 1) / 20) * 20;
    for chunk in input[..body_len].chunks_exact(20) {
        let a0 = read_u32(chunk, 0)
            .wrapping_mul(C1)
            .rotate_right(17)
            .wrapping_mul(C2);
        let a1 = read_u32(chunk, 4);
        let a2 = read_u32(chunk, 8)
            .wrapping_mul(C1)
            .rotate_right(17)
            .wrapping_mul(C2);
        let a3 = read_u32(chunk, 12)
            .wrapping_mul(C1)
            .rotate_right(17)
            .wrapping_mul(C2);
        let a4 = read_u32(chunk, 16);

        hash ^= a0;
        hash = hash
            .rotate_right(18)
            .wrapping_mul(5)
            .wrapping_add(0xe654_6b64);
        f = f.wrapping_add(a1).rotate_right(19).wrapping_mul(C1);
        g = g
            .wrapping_add(a2)
            .rotate_right(18)
            .wrapping_mul(5)
            .wrapping_add(0xe654_6b64);
        hash ^= a3.wrapping_add(a1);
        hash = hash
            .rotate_right(19)
            .wrapping_mul(5)
            .wrapping_add(0xe654_6b64);
        g ^= a4;
        g = g.swap_bytes().wrapping_mul(5);
        hash = hash.wrapping_add(a4.wrapping_mul(5)).swap_bytes();
        f = f.wrapping_add(a0);
        (f, hash, g) = (g, f, hash);
    }

    g = g.rotate_right(11).wrapping_mul(C1);
    g = g.rotate_right(17).wrapping_mul(C1);
    f = f.rotate_right(11).wrapping_mul(C1);
    f = f.rotate_right(17).wrapping_mul(C1);
    hash = hash
        .wrapping_add(g)
        .rotate_right(19)
        .wrapping_mul(5)
        .wrapping_add(0xe654_6b64)
        .rotate_right(17)
        .wrapping_mul(C1);
    hash.wrapping_add(f)
        .rotate_right(19)
        .wrapping_mul(5)
        .wrapping_add(0xe654_6b64)
        .rotate_right(17)
        .wrapping_mul(C1)
}

#[inline(always)]
fn shift_mix(value: u64) -> u64 {
    value ^ (value >> 47)
}

#[inline(always)]
fn hash_len_16(left: u64, right: u64) -> u64 {
    let mut a = (left ^ right).wrapping_mul(HASH128_MUL);
    a ^= a >> 47;
    let mut b = (right ^ a).wrapping_mul(HASH128_MUL);
    b ^= b >> 47;
    b.wrapping_mul(HASH128_MUL)
}

#[inline(always)]
fn hash_len_16_with_mul(left: u64, right: u64, mul: u64) -> u64 {
    let mut a = (left ^ right).wrapping_mul(mul);
    a ^= a >> 47;
    let mut b = (right ^ a).wrapping_mul(mul);
    b ^= b >> 47;
    b.wrapping_mul(mul)
}

#[inline]
fn hash64_len_0_to_16(input: &[u8]) -> u64 {
    let len = input.len();
    if len >= 8 {
        let mul = K2.wrapping_add((len as u64).wrapping_mul(2));
        let a = read_u64(input, 0).wrapping_add(K2);
        let b = read_u64(input, len - 8);
        let c = b.rotate_right(37).wrapping_mul(mul).wrapping_add(a);
        let d = a.rotate_right(25).wrapping_add(b).wrapping_mul(mul);
        hash_len_16_with_mul(c, d, mul)
    } else if len >= 4 {
        let mul = K2.wrapping_add((len as u64).wrapping_mul(2));
        let a = u64::from(read_u32(input, 0));
        hash_len_16_with_mul(
            (len as u64).wrapping_add(a << 3),
            u64::from(read_u32(input, len - 4)),
            mul,
        )
    } else if len > 0 {
        let a = u32::from(input[0]);
        let b = u32::from(input[len >> 1]);
        let c = u32::from(input[len - 1]);
        let y = a.wrapping_add(b << 8);
        let z = (len as u32).wrapping_add(c << 2);
        shift_mix(u64::from(y).wrapping_mul(K2) ^ u64::from(z).wrapping_mul(K0)).wrapping_mul(K2)
    } else {
        K2
    }
}

#[inline]
fn hash64_len_17_to_32(input: &[u8]) -> u64 {
    let len = input.len();
    let mul = K2.wrapping_add((len as u64).wrapping_mul(2));
    let a = read_u64(input, 0).wrapping_mul(K1);
    let b = read_u64(input, 8);
    let c = read_u64(input, len - 8).wrapping_mul(mul);
    let d = read_u64(input, len - 16).wrapping_mul(K2);
    hash_len_16_with_mul(
        a.wrapping_add(b)
            .rotate_right(43)
            .wrapping_add(c.rotate_right(30))
            .wrapping_add(d),
        a.wrapping_add(b.wrapping_add(K2).rotate_right(18))
            .wrapping_add(c),
        mul,
    )
}

#[inline]
fn weak_hash_len_32_with_seeds(input: &[u8], mut a: u64, mut b: u64) -> (u64, u64) {
    assert!(input.len() >= 32, "CityHash block must contain 32 bytes");
    let w = read_u64(input, 0);
    let x = read_u64(input, 8);
    let y = read_u64(input, 16);
    let z = read_u64(input, 24);
    a = a.wrapping_add(w);
    b = b.wrapping_add(a).wrapping_add(z).rotate_right(21);
    let c = a;
    a = a.wrapping_add(x).wrapping_add(y);
    b = b.wrapping_add(a.rotate_right(44));
    (a.wrapping_add(z), b.wrapping_add(c))
}

#[inline]
fn hash64_len_33_to_64(input: &[u8]) -> u64 {
    let len = input.len();
    let mul = K2.wrapping_add((len as u64).wrapping_mul(2));
    let mut a = read_u64(input, 0).wrapping_mul(K2);
    let mut b = read_u64(input, 8);
    let c = read_u64(input, len - 24);
    let d = read_u64(input, len - 32);
    let e = read_u64(input, 16).wrapping_mul(K2);
    let f = read_u64(input, 24).wrapping_mul(9);
    let g = read_u64(input, len - 8);
    let h = read_u64(input, len - 16).wrapping_mul(mul);
    let u = a
        .wrapping_add(g)
        .rotate_right(43)
        .wrapping_add(b.rotate_right(30).wrapping_add(c).wrapping_mul(9));
    let v = (a.wrapping_add(g) ^ d).wrapping_add(f).wrapping_add(1);
    let w = u
        .wrapping_add(v)
        .wrapping_mul(mul)
        .swap_bytes()
        .wrapping_add(h);
    let x = e.wrapping_add(f).rotate_right(42).wrapping_add(c);
    let y = v
        .wrapping_add(w)
        .wrapping_mul(mul)
        .swap_bytes()
        .wrapping_add(g)
        .wrapping_mul(mul);
    let z = e.wrapping_add(f).wrapping_add(c);
    a = x
        .wrapping_add(z)
        .wrapping_mul(mul)
        .wrapping_add(y)
        .swap_bytes()
        .wrapping_add(b);
    b = shift_mix(
        z.wrapping_add(a)
            .wrapping_mul(mul)
            .wrapping_add(d)
            .wrapping_add(h),
    )
    .wrapping_mul(mul);
    b.wrapping_add(x)
}

/// Hashes `input` with CityHash64 1.1.1.
#[must_use]
#[inline]
pub fn cityhash64(input: &[u8]) -> u64 {
    let len = input.len();
    if len <= 32 {
        return if len <= 16 {
            hash64_len_0_to_16(input)
        } else {
            hash64_len_17_to_32(input)
        };
    }
    if len <= 64 {
        return hash64_len_33_to_64(input);
    }

    let mut x = read_u64(input, len - 40);
    let mut y = read_u64(input, len - 16).wrapping_add(read_u64(input, len - 56));
    let mut z = hash_len_16(
        read_u64(input, len - 48).wrapping_add(len as u64),
        read_u64(input, len - 24),
    );
    let mut v = weak_hash_len_32_with_seeds(&input[len - 64..], len as u64, z);
    let mut w = weak_hash_len_32_with_seeds(&input[len - 32..], y.wrapping_add(K1), x);
    x = x.wrapping_mul(K1).wrapping_add(read_u64(input, 0));

    for chunk in input[..len - 1].chunks_exact(64) {
        x = x
            .wrapping_add(y)
            .wrapping_add(v.0)
            .wrapping_add(read_u64(chunk, 8))
            .rotate_right(37)
            .wrapping_mul(K1);
        y = y
            .wrapping_add(v.1)
            .wrapping_add(read_u64(chunk, 48))
            .rotate_right(42)
            .wrapping_mul(K1);
        x ^= w.1;
        y = y.wrapping_add(v.0).wrapping_add(read_u64(chunk, 40));
        z = z.wrapping_add(w.0).rotate_right(33).wrapping_mul(K1);
        v = weak_hash_len_32_with_seeds(chunk, v.1.wrapping_mul(K1), x.wrapping_add(w.0));
        w = weak_hash_len_32_with_seeds(
            &chunk[32..],
            z.wrapping_add(w.1),
            y.wrapping_add(read_u64(chunk, 16)),
        );
        core::mem::swap(&mut z, &mut x);
    }

    hash_len_16(
        hash_len_16(v.0, w.0)
            .wrapping_add(shift_mix(y).wrapping_mul(K1))
            .wrapping_add(z),
        hash_len_16(v.1, w.1).wrapping_add(x),
    )
}

/// Hashes `input` with CityHash64 1.1.1 and one 64-bit seed.
#[must_use]
#[inline]
pub fn cityhash64_with_seed(input: &[u8], seed: u64) -> u64 {
    cityhash64_with_seeds(input, K2, seed)
}

/// Hashes `input` with CityHash64 1.1.1 and two 64-bit seeds.
#[must_use]
#[inline]
pub fn cityhash64_with_seeds(input: &[u8], seed0: u64, seed1: u64) -> u64 {
    hash_len_16(cityhash64(input).wrapping_sub(seed0), seed1)
}

#[inline]
fn city_murmur(input: &[u8], seed: (u64, u64)) -> (u64, u64) {
    let mut a = seed.0;
    let mut b = seed.1;
    let mut c;
    let mut d;
    let len = input.len();

    if len <= 16 {
        a = shift_mix(a.wrapping_mul(K1)).wrapping_mul(K1);
        c = b.wrapping_mul(K1).wrapping_add(hash64_len_0_to_16(input));
        d = shift_mix(a.wrapping_add(if len >= 8 { read_u64(input, 0) } else { c }));
    } else {
        c = hash_len_16(read_u64(input, len - 8).wrapping_add(K1), a);
        d = hash_len_16(
            b.wrapping_add(len as u64),
            c.wrapping_add(read_u64(input, len - 16)),
        );
        a = a.wrapping_add(d);

        let mut offset = 0;
        let mut remaining = len - 16;
        loop {
            a ^= shift_mix(read_u64(input, offset).wrapping_mul(K1)).wrapping_mul(K1);
            a = a.wrapping_mul(K1);
            b ^= a;
            c ^= shift_mix(read_u64(input, offset + 8).wrapping_mul(K1)).wrapping_mul(K1);
            c = c.wrapping_mul(K1);
            d ^= c;
            offset += 16;
            if remaining <= 16 {
                break;
            }
            remaining -= 16;
        }
    }

    a = hash_len_16(a, c);
    b = hash_len_16(d, b);
    (a ^ b, hash_len_16(b, a))
}

#[inline]
fn cityhash128_with_seed_parts(input: &[u8], seed: (u64, u64)) -> (u64, u64) {
    let len = input.len();
    if len < 128 {
        return city_murmur(input, seed);
    }

    let mut x = seed.0;
    let mut y = seed.1;
    let mut z = (len as u64).wrapping_mul(K1);
    let mut v = (
        (y ^ K1)
            .rotate_right(49)
            .wrapping_mul(K1)
            .wrapping_add(read_u64(input, 0)),
        0,
    );
    v.1 =
        v.0.rotate_right(42)
            .wrapping_mul(K1)
            .wrapping_add(read_u64(input, 8));
    let mut w = (
        y.wrapping_add(z)
            .rotate_right(35)
            .wrapping_mul(K1)
            .wrapping_add(x),
        x.wrapping_add(read_u64(input, 88))
            .rotate_right(53)
            .wrapping_mul(K1),
    );

    let mut blocks = input.chunks_exact(128);
    for block in &mut blocks {
        for chunk in block.chunks_exact(64) {
            x = x
                .wrapping_add(y)
                .wrapping_add(v.0)
                .wrapping_add(read_u64(chunk, 8))
                .rotate_right(37)
                .wrapping_mul(K1);
            y = y
                .wrapping_add(v.1)
                .wrapping_add(read_u64(chunk, 48))
                .rotate_right(42)
                .wrapping_mul(K1);
            x ^= w.1;
            y = y.wrapping_add(v.0).wrapping_add(read_u64(chunk, 40));
            z = z.wrapping_add(w.0).rotate_right(33).wrapping_mul(K1);
            v = weak_hash_len_32_with_seeds(chunk, v.1.wrapping_mul(K1), x.wrapping_add(w.0));
            w = weak_hash_len_32_with_seeds(
                &chunk[32..],
                z.wrapping_add(w.1),
                y.wrapping_add(read_u64(chunk, 16)),
            );
            core::mem::swap(&mut z, &mut x);
        }
    }
    let remaining = blocks.remainder().len();
    let offset = len - remaining;

    x = x.wrapping_add(v.0.wrapping_add(z).rotate_right(49).wrapping_mul(K0));
    y = y.wrapping_mul(K0).wrapping_add(w.1.rotate_right(37));
    z = z.wrapping_mul(K0).wrapping_add(w.0.rotate_right(27));
    w.0 = w.0.wrapping_mul(9);
    v.0 = v.0.wrapping_mul(K0);

    let mut tail_done = 0;
    while tail_done < remaining {
        tail_done += 32;
        let tail = offset + remaining - tail_done;
        y = x
            .wrapping_add(y)
            .rotate_right(42)
            .wrapping_mul(K0)
            .wrapping_add(v.1);
        w.0 = w.0.wrapping_add(read_u64(input, tail + 16));
        x = x.wrapping_mul(K0).wrapping_add(w.0);
        z = z.wrapping_add(w.1).wrapping_add(read_u64(input, tail));
        w.1 = w.1.wrapping_add(v.0);
        v = weak_hash_len_32_with_seeds(&input[tail..], v.0.wrapping_add(z), v.1);
        v.0 = v.0.wrapping_mul(K0);
    }

    x = hash_len_16(x, v.0);
    y = hash_len_16(y.wrapping_add(z), w.0);
    (
        hash_len_16(x.wrapping_add(v.1), w.1).wrapping_add(y),
        hash_len_16(x.wrapping_add(w.1), y.wrapping_add(v.1)),
    )
}

#[inline(always)]
fn parts_to_u128((low, high): (u64, u64)) -> u128 {
    (u128::from(high) << 64) | u128::from(low)
}

#[inline(always)]
fn u128_to_parts(value: u128) -> (u64, u64) {
    (value as u64, (value >> 64) as u64)
}

/// Hashes `input` with CityHash128 1.1.1.
///
/// The returned integer stores the reference algorithm's high 64-bit word in
/// the most significant half and its low word in the least significant half.
#[must_use]
#[inline]
pub fn cityhash128(input: &[u8]) -> u128 {
    let parts = if input.len() >= 16 {
        cityhash128_with_seed_parts(
            &input[16..],
            (read_u64(input, 0), read_u64(input, 8).wrapping_add(K0)),
        )
    } else {
        cityhash128_with_seed_parts(input, (K0, K1))
    };
    parts_to_u128(parts)
}

/// Hashes `input` with CityHash128 1.1.1 and a 128-bit seed.
///
/// `seed` and the returned digest both store the high 64-bit word in the most
/// significant half and the low word in the least significant half.
#[must_use]
#[inline]
pub fn cityhash128_with_seed(input: &[u8], seed: u128) -> u128 {
    parts_to_u128(cityhash128_with_seed_parts(input, u128_to_parts(seed)))
}

/// Hashes a 128-bit value down to 64 bits using CityHash's public reducer.
#[must_use]
#[inline]
pub fn cityhash128_to_64(value: u128) -> u64 {
    let (low, high) = u128_to_parts(value);
    hash_len_16(low, high)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn official_empty_vectors_match() {
        assert_eq!(cityhash32(b""), 0xdc56_d17a);
        assert_eq!(cityhash64(b""), 0x9ae1_6a3b_2f90_404f);
        assert_eq!(cityhash128(b""), 0x3cb5_40c3_92e5_1e29_3df0_9dfc_64c0_9a2b);
    }

    #[test]
    fn official_seeded_vectors_match() {
        let input = [0, 1, 2, 3, 4];
        assert_eq!(cityhash32(&input), 0xfe6e_37d4);
        assert_eq!(cityhash64(&input), 0xb4bf_a9e8_7732_c149);
        assert_eq!(cityhash64_with_seed(&input, 123), 0xce17_0601_9c5e_61a7);
        assert_eq!(
            cityhash64_with_seeds(&input, 123, 456),
            0xd83c_8188_1e3a_35e3
        );
        assert_eq!(
            cityhash128(&input),
            0xe3cb_1f3f_3ab9_643b_ef36_68c1_5001_2eec
        );
        assert_eq!(
            cityhash128_with_seed(&input, 123),
            0x68da_6334_de1f_04c9_ce25_5b96_13ad_58b7
        );
        assert_eq!(
            cityhash128_to_64(0x68da_6334_de1f_04c9_ce25_5b96_13ad_58b7),
            0xa991_2800_57f9_aaca
        );
    }
}
