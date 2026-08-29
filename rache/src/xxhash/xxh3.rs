// Copyright 2026 rache contributors
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

//! XXH3-64 and XXH3-128 one-shot and streaming APIs.

use core::fmt;
use core::hash::{BuildHasher, Hasher};

use super::kernel::{self, Xxh3Kernel};
use crate::util::{mul128_fold64, read_u32, read_u64};

const PRIME32_1: u64 = 0x9e37_79b1;
const PRIME32_2: u64 = 0x85eb_ca77;
const PRIME32_3: u64 = 0xc2b2_ae3d;
const PRIME64_1: u64 = 0x9e37_79b1_85eb_ca87;
const PRIME64_2: u64 = 0xc2b2_ae3d_27d4_eb4f;
const PRIME64_3: u64 = 0x1656_67b1_9e37_79f9;
const PRIME64_4: u64 = 0x85eb_ca77_c2b2_ae63;
const PRIME64_5: u64 = 0x27d4_eb2f_1656_67c5;
const PRIME_MX1: u64 = 0x1656_6791_9e37_79f9;
const PRIME_MX2: u64 = 0x9fb2_1c65_1e98_df25;

/// The minimum number of bytes accepted by the XXH3 custom-secret APIs.
pub const SECRET_SIZE_MIN: usize = 136;
/// The number of bytes in the standard XXH3 secret.
pub const DEFAULT_SECRET_SIZE: usize = 192;
const STRIPE_SIZE: usize = 64;
const SECRET_CONSUME_RATE: usize = 8;
const MIDSIZE_MAX: usize = 240;
const STREAM_BUFFER_SIZE: usize = STRIPE_SIZE * 4;
const SECRET_LAST_ACC_START: usize = 7;
const SECRET_MERGE_ACC_START: usize = 11;

#[rustfmt::skip]
const INITIAL_ACC: [u64; 8] = [
    PRIME32_3, PRIME64_1, PRIME64_2, PRIME64_3,
    PRIME64_4, PRIME32_2, PRIME64_5, PRIME32_1,
];

/// The standard 192-byte XXH3 secret.
pub const DEFAULT_SECRET: [u8; DEFAULT_SECRET_SIZE] = [
    0xb8, 0xfe, 0x6c, 0x39, 0x23, 0xa4, 0x4b, 0xbe, 0x7c, 0x01, 0x81, 0x2c, 0xf7, 0x21, 0xad, 0x1c,
    0xde, 0xd4, 0x6d, 0xe9, 0x83, 0x90, 0x97, 0xdb, 0x72, 0x40, 0xa4, 0xa4, 0xb7, 0xb3, 0x67, 0x1f,
    0xcb, 0x79, 0xe6, 0x4e, 0xcc, 0xc0, 0xe5, 0x78, 0x82, 0x5a, 0xd0, 0x7d, 0xcc, 0xff, 0x72, 0x21,
    0xb8, 0x08, 0x46, 0x74, 0xf7, 0x43, 0x24, 0x8e, 0xe0, 0x35, 0x90, 0xe6, 0x81, 0x3a, 0x26, 0x4c,
    0x3c, 0x28, 0x52, 0xbb, 0x91, 0xc3, 0x00, 0xcb, 0x88, 0xd0, 0x65, 0x8b, 0x1b, 0x53, 0x2e, 0xa3,
    0x71, 0x64, 0x48, 0x97, 0xa2, 0x0d, 0xf9, 0x4e, 0x38, 0x19, 0xef, 0x46, 0xa9, 0xde, 0xac, 0xd8,
    0xa8, 0xfa, 0x76, 0x3f, 0xe3, 0x9c, 0x34, 0x3f, 0xf9, 0xdc, 0xbb, 0xc7, 0xc7, 0x0b, 0x4f, 0x1d,
    0x8a, 0x51, 0xe0, 0x4b, 0xcd, 0xb4, 0x59, 0x31, 0xc8, 0x9f, 0x7e, 0xc9, 0xd9, 0x78, 0x73, 0x64,
    0xea, 0xc5, 0xac, 0x83, 0x34, 0xd3, 0xeb, 0xc3, 0xc5, 0x81, 0xa0, 0xff, 0xfa, 0x13, 0x63, 0xeb,
    0x17, 0x0d, 0xdd, 0x51, 0xb7, 0xf0, 0xda, 0x49, 0xd3, 0x16, 0x55, 0x26, 0x29, 0xd4, 0x68, 0x9e,
    0x2b, 0x16, 0xbe, 0x58, 0x7d, 0x47, 0xa1, 0xfc, 0x8f, 0xf8, 0xb8, 0xd1, 0x7a, 0xd0, 0x31, 0xce,
    0x45, 0xcb, 0x3a, 0x8f, 0x95, 0x16, 0x04, 0x28, 0xaf, 0xd7, 0xfb, 0xca, 0xbb, 0x4b, 0x40, 0x7e,
];

/// Error returned when an XXH3 custom secret is shorter than 136 bytes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Xxh3SecretTooShort {
    actual_len: usize,
}

impl Xxh3SecretTooShort {
    /// Returns the supplied secret length.
    #[must_use]
    pub const fn actual_len(self) -> usize {
        self.actual_len
    }

    /// Returns the minimum accepted secret length.
    #[must_use]
    pub const fn minimum_len() -> usize {
        SECRET_SIZE_MIN
    }
}

impl fmt::Display for Xxh3SecretTooShort {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "XXH3 secret is {} bytes; at least {SECRET_SIZE_MIN} bytes are required",
            self.actual_len
        )
    }
}

impl core::error::Error for Xxh3SecretTooShort {}

#[inline]
fn validate_secret(secret: &[u8]) -> Result<(), Xxh3SecretTooShort> {
    if secret.len() < SECRET_SIZE_MIN {
        Err(Xxh3SecretTooShort {
            actual_len: secret.len(),
        })
    } else {
        Ok(())
    }
}

/// Hashes `input` with unseeded XXH3-64.
#[must_use]
#[inline]
pub fn xxh3_64(input: &[u8]) -> u64 {
    if input.len() <= MIDSIZE_MAX {
        hash_64_short(input, 0, &DEFAULT_SECRET)
    } else {
        kernel::dispatch!(hash_long_64(input, &DEFAULT_SECRET))
    }
}

/// Hashes `input` with seeded XXH3-64.
#[must_use]
#[inline]
pub fn xxh3_64_with_seed(input: &[u8], seed: u64) -> u64 {
    if seed == 0 {
        xxh3_64(input)
    } else if input.len() <= MIDSIZE_MAX {
        hash_64_short(input, seed, &DEFAULT_SECRET)
    } else {
        let secret = derive_secret(seed);
        kernel::dispatch!(hash_long_64(input, &secret))
    }
}

/// Hashes `input` with an XXH3-64 custom secret.
///
/// `secret` must contain at least [`SECRET_SIZE_MIN`] bytes and should contain
/// high-entropy data. A custom secret changes the digest but does not make
/// XXH3 cryptographically secure.
pub fn xxh3_64_with_secret(input: &[u8], secret: &[u8]) -> Result<u64, Xxh3SecretTooShort> {
    validate_secret(secret)?;
    Ok(if input.len() <= MIDSIZE_MAX {
        hash_64_short(input, 0, secret)
    } else {
        kernel::dispatch!(hash_long_64(input, secret))
    })
}

/// Hashes `input` with an XXH3-64 seed and custom secret.
///
/// This follows the reference XXH3 contract: inputs up to 240 bytes use
/// `seed`, while longer inputs use `secret`.
pub fn xxh3_64_with_seed_and_secret(
    input: &[u8],
    seed: u64,
    secret: &[u8],
) -> Result<u64, Xxh3SecretTooShort> {
    validate_secret(secret)?;
    Ok(if input.len() <= MIDSIZE_MAX {
        hash_64_short(input, seed, &DEFAULT_SECRET)
    } else {
        kernel::dispatch!(hash_long_64(input, secret))
    })
}

/// Hashes `input` with unseeded XXH3-128.
///
/// The returned integer stores the reference algorithm's high 64 bits in the
/// most significant half and its low 64 bits in the least significant half.
#[must_use]
#[inline]
pub fn xxh3_128(input: &[u8]) -> u128 {
    if input.len() <= MIDSIZE_MAX {
        hash_128_short(input, 0, &DEFAULT_SECRET)
    } else {
        kernel::dispatch!(hash_long_128(input, &DEFAULT_SECRET))
    }
}

/// Hashes `input` with seeded XXH3-128.
#[must_use]
#[inline]
pub fn xxh3_128_with_seed(input: &[u8], seed: u64) -> u128 {
    if seed == 0 {
        xxh3_128(input)
    } else if input.len() <= MIDSIZE_MAX {
        hash_128_short(input, seed, &DEFAULT_SECRET)
    } else {
        let secret = derive_secret(seed);
        kernel::dispatch!(hash_long_128(input, &secret))
    }
}

/// Hashes `input` with an XXH3-128 custom secret.
///
/// `secret` must contain at least [`SECRET_SIZE_MIN`] bytes and should contain
/// high-entropy data. A custom secret changes the digest but does not make
/// XXH3 cryptographically secure.
pub fn xxh3_128_with_secret(input: &[u8], secret: &[u8]) -> Result<u128, Xxh3SecretTooShort> {
    validate_secret(secret)?;
    Ok(if input.len() <= MIDSIZE_MAX {
        hash_128_short(input, 0, secret)
    } else {
        kernel::dispatch!(hash_long_128(input, secret))
    })
}

/// Hashes `input` with an XXH3-128 seed and custom secret.
///
/// This follows the reference XXH3 contract: inputs up to 240 bytes use
/// `seed`, while longer inputs use `secret`.
pub fn xxh3_128_with_seed_and_secret(
    input: &[u8],
    seed: u64,
    secret: &[u8],
) -> Result<u128, Xxh3SecretTooShort> {
    validate_secret(secret)?;
    Ok(if input.len() <= MIDSIZE_MAX {
        hash_128_short(input, seed, &DEFAULT_SECRET)
    } else {
        kernel::dispatch!(hash_long_128(input, secret))
    })
}

#[inline]
fn derive_secret(seed: u64) -> [u8; DEFAULT_SECRET_SIZE] {
    let mut secret = DEFAULT_SECRET;
    if seed == 0 {
        return secret;
    }

    for offset in (0..DEFAULT_SECRET_SIZE).step_by(16) {
        let low = read_u64(&secret, offset).wrapping_add(seed);
        let high = read_u64(&secret, offset + 8).wrapping_sub(seed);
        secret[offset..offset + 8].copy_from_slice(&low.to_le_bytes());
        secret[offset + 8..offset + 16].copy_from_slice(&high.to_le_bytes());
    }
    secret
}

#[inline(always)]
fn avalanche(mut value: u64) -> u64 {
    value ^= value >> 37;
    value = value.wrapping_mul(PRIME_MX1);
    value ^ (value >> 32)
}

#[inline(always)]
fn avalanche_xxh64(mut value: u64) -> u64 {
    value ^= value >> 33;
    value = value.wrapping_mul(PRIME64_2);
    value ^= value >> 29;
    value = value.wrapping_mul(PRIME64_3);
    value ^ (value >> 32)
}

#[inline(always)]
fn rrmxmx(mut value: u64, len: usize) -> u64 {
    value ^= value.rotate_left(49) ^ value.rotate_left(24);
    value = value.wrapping_mul(PRIME_MX2);
    value ^= (value >> 35).wrapping_add(len as u64);
    value = value.wrapping_mul(PRIME_MX2);
    value ^ (value >> 28)
}

#[inline(always)]
fn combined_1_to_3(input: &[u8]) -> u32 {
    u32::from(input[input.len() - 1])
        | ((input.len() as u32) << 8)
        | (u32::from(input[0]) << 16)
        | (u32::from(input[input.len() >> 1]) << 24)
}

#[inline(always)]
fn mix_16(
    input: &[u8],
    input_offset: usize,
    secret: &[u8],
    secret_offset: usize,
    seed: u64,
) -> u64 {
    let low = read_u64(input, input_offset) ^ read_u64(secret, secret_offset).wrapping_add(seed);
    let high =
        read_u64(input, input_offset + 8) ^ read_u64(secret, secret_offset + 8).wrapping_sub(seed);
    mul128_fold64(low, high)
}

#[inline(always)]
fn hash_64_short(input: &[u8], seed: u64, secret: &[u8]) -> u64 {
    match input.len() {
        0 => {
            let bitflip = read_u64(secret, 56) ^ read_u64(secret, 64);
            avalanche_xxh64(seed ^ bitflip)
        }
        1..=3 => {
            let bitflip = u64::from(read_u32(secret, 0) ^ read_u32(secret, 4));
            avalanche_xxh64(u64::from(combined_1_to_3(input)) ^ bitflip.wrapping_add(seed))
        }
        4..=8 => {
            let modified_seed = seed ^ (u64::from((seed as u32).swap_bytes()) << 32);
            let input64 =
                u64::from(read_u32(input, input.len() - 4)) | (u64::from(read_u32(input, 0)) << 32);
            let bitflip = (read_u64(secret, 8) ^ read_u64(secret, 16)).wrapping_sub(modified_seed);
            rrmxmx(input64 ^ bitflip, input.len())
        }
        9..=16 => {
            let low = read_u64(input, 0)
                ^ (read_u64(secret, 24) ^ read_u64(secret, 32)).wrapping_add(seed);
            let high = read_u64(input, input.len() - 8)
                ^ (read_u64(secret, 40) ^ read_u64(secret, 48)).wrapping_sub(seed);
            let value = (input.len() as u64)
                .wrapping_add(low.swap_bytes())
                .wrapping_add(high)
                .wrapping_add(mul128_fold64(low, high));
            avalanche(value)
        }
        17..=128 => hash_64_17_to_128(input, seed, secret),
        129..=240 => hash_64_129_to_240(input, seed, secret),
        _ => unreachable!("short-input dispatch validates the length"),
    }
}

#[inline]
fn hash_64_17_to_128(input: &[u8], seed: u64, secret: &[u8]) -> u64 {
    let len = input.len();
    let mut acc = (len as u64).wrapping_mul(PRIME64_1);

    acc = acc.wrapping_add(mix_16(input, 0, secret, 0, seed));
    acc = acc.wrapping_add(mix_16(input, len - 16, secret, 16, seed));
    if len > 32 {
        acc = acc.wrapping_add(mix_16(input, 16, secret, 32, seed));
        acc = acc.wrapping_add(mix_16(input, len - 32, secret, 48, seed));
    }
    if len > 64 {
        acc = acc.wrapping_add(mix_16(input, 32, secret, 64, seed));
        acc = acc.wrapping_add(mix_16(input, len - 48, secret, 80, seed));
    }
    if len > 96 {
        acc = acc.wrapping_add(mix_16(input, 48, secret, 96, seed));
        acc = acc.wrapping_add(mix_16(input, len - 64, secret, 112, seed));
    }
    avalanche(acc)
}

#[inline]
fn hash_64_129_to_240(input: &[u8], seed: u64, secret: &[u8]) -> u64 {
    let mut acc = (input.len() as u64).wrapping_mul(PRIME64_1);
    for chunk in 0..8 {
        acc = acc.wrapping_add(mix_16(input, chunk * 16, secret, chunk * 16, seed));
    }
    acc = avalanche(acc);

    let chunk_count = input.len() / 16;
    for chunk in 8..chunk_count {
        acc = acc.wrapping_add(mix_16(
            input,
            chunk * 16,
            secret,
            3 + (chunk - 8) * 16,
            seed,
        ));
    }
    acc = acc.wrapping_add(mix_16(input, input.len() - 16, secret, 119, seed));
    avalanche(acc)
}

#[inline(always)]
fn hash_128_short(input: &[u8], seed: u64, secret: &[u8]) -> u128 {
    match input.len() {
        0 => {
            let low = avalanche_xxh64(seed ^ read_u64(secret, 64) ^ read_u64(secret, 72));
            let high = avalanche_xxh64(seed ^ read_u64(secret, 80) ^ read_u64(secret, 88));
            make_u128(low, high)
        }
        1..=3 => {
            let combined = combined_1_to_3(input);
            let low_bitflip = u64::from(read_u32(secret, 0) ^ read_u32(secret, 4));
            let high_bitflip = u64::from(read_u32(secret, 8) ^ read_u32(secret, 12));
            let low = avalanche_xxh64(u64::from(combined) ^ low_bitflip.wrapping_add(seed));
            let high_input = combined.swap_bytes().rotate_left(13);
            let high = avalanche_xxh64(u64::from(high_input) ^ high_bitflip.wrapping_sub(seed));
            make_u128(low, high)
        }
        4..=8 => hash_128_4_to_8(input, seed, secret),
        9..=16 => hash_128_9_to_16(input, seed, secret),
        17..=128 => hash_128_17_to_128(input, seed, secret),
        129..=240 => hash_128_129_to_240(input, seed, secret),
        _ => unreachable!("short-input dispatch validates the length"),
    }
}

#[inline(always)]
fn hash_128_4_to_8(input: &[u8], seed: u64, secret: &[u8]) -> u128 {
    let modified_seed = seed ^ (u64::from((seed as u32).swap_bytes()) << 32);
    let input64 =
        u64::from(read_u32(input, 0)) | (u64::from(read_u32(input, input.len() - 4)) << 32);
    let bitflip = (read_u64(secret, 16) ^ read_u64(secret, 24)).wrapping_add(modified_seed);
    let product = u128::from(input64 ^ bitflip).wrapping_mul(u128::from(
        PRIME64_1.wrapping_add((input.len() as u64) << 2),
    ));
    let mut low = product as u64;
    let mut high = (product >> 64) as u64;
    high = high.wrapping_add(low << 1);
    low ^= high >> 3;
    low ^= low >> 35;
    low = low.wrapping_mul(PRIME_MX2);
    low ^= low >> 28;
    make_u128(low, avalanche(high))
}

#[inline(always)]
fn hash_128_9_to_16(input: &[u8], seed: u64, secret: &[u8]) -> u128 {
    let first = read_u64(input, 0);
    let last = read_u64(input, input.len() - 8);
    let value1 = first ^ last ^ (read_u64(secret, 32) ^ read_u64(secret, 40)).wrapping_sub(seed);
    let value2 = last ^ (read_u64(secret, 48) ^ read_u64(secret, 56)).wrapping_add(seed);
    let product = u128::from(value1).wrapping_mul(u128::from(PRIME64_1));
    let mut low = (product as u64).wrapping_add(((input.len() - 1) as u64) << 54);
    let high = ((product >> 64) as u64)
        .wrapping_add((value2 >> 32) << 32)
        .wrapping_add(u64::from(value2 as u32).wrapping_mul(PRIME32_2));
    low ^= high.swap_bytes();
    let product = make_u128(low, high).wrapping_mul(u128::from(PRIME64_2));
    make_u128(avalanche(product as u64), avalanche((product >> 64) as u64))
}

#[inline]
fn mix_32(
    acc: &mut [u64; 2],
    input: &[u8],
    first_offset: usize,
    second_offset: usize,
    secret: &[u8],
    secret_offset: usize,
    seed: u64,
) {
    acc[0] = acc[0].wrapping_add(mix_16(input, first_offset, secret, secret_offset, seed));
    acc[1] = acc[1].wrapping_add(mix_16(
        input,
        second_offset,
        secret,
        secret_offset + 16,
        seed,
    ));
    acc[0] ^= read_u64(input, second_offset).wrapping_add(read_u64(input, second_offset + 8));
    acc[1] ^= read_u64(input, first_offset).wrapping_add(read_u64(input, first_offset + 8));
}

#[inline(always)]
fn finalize_128_medium(acc: [u64; 2], len: u64, seed: u64) -> u128 {
    let low = avalanche(acc[0].wrapping_add(acc[1]));
    let high = acc[0]
        .wrapping_mul(PRIME64_1)
        .wrapping_add(acc[1].wrapping_mul(PRIME64_4))
        .wrapping_add(len.wrapping_sub(seed).wrapping_mul(PRIME64_2));
    make_u128(low, avalanche(high).wrapping_neg())
}

#[inline]
fn hash_128_17_to_128(input: &[u8], seed: u64, secret: &[u8]) -> u128 {
    let len = input.len();
    let mut acc = [(len as u64).wrapping_mul(PRIME64_1), 0];
    if len > 96 {
        mix_32(&mut acc, input, 48, len - 64, secret, 96, seed);
    }
    if len > 64 {
        mix_32(&mut acc, input, 32, len - 48, secret, 64, seed);
    }
    if len > 32 {
        mix_32(&mut acc, input, 16, len - 32, secret, 32, seed);
    }
    mix_32(&mut acc, input, 0, len - 16, secret, 0, seed);
    finalize_128_medium(acc, len as u64, seed)
}

#[inline]
fn hash_128_129_to_240(input: &[u8], seed: u64, secret: &[u8]) -> u128 {
    let len = input.len() as u64;
    let mut acc = [len.wrapping_mul(PRIME64_1), 0];
    for pair in 0..4 {
        mix_32(
            &mut acc,
            input,
            pair * 32,
            pair * 32 + 16,
            secret,
            pair * 32,
            seed,
        );
    }
    acc = acc.map(avalanche);

    let pair_count = input.len() / 32;
    for pair in 4..pair_count {
        mix_32(
            &mut acc,
            input,
            pair * 32,
            pair * 32 + 16,
            secret,
            3 + (pair - 4) * 32,
            seed,
        );
    }
    mix_32(
        &mut acc,
        input,
        input.len() - 16,
        input.len() - 32,
        secret,
        103,
        seed.wrapping_neg(),
    );
    finalize_128_medium(acc, len, seed)
}

#[inline(always)]
const fn make_u128(low: u64, high: u64) -> u128 {
    ((high as u128) << 64) | low as u128
}

#[inline]
fn hash_long_64<K: Xxh3Kernel>(kernel: K, input: &[u8], secret: &[u8]) -> u64 {
    let acc = accumulate_long(kernel, input, secret);
    merge_accumulators(
        &acc,
        (input.len() as u64).wrapping_mul(PRIME64_1),
        secret,
        SECRET_MERGE_ACC_START,
    )
}

#[inline]
fn hash_long_128<K: Xxh3Kernel>(kernel: K, input: &[u8], secret: &[u8]) -> u128 {
    let acc = accumulate_long(kernel, input, secret);
    let len = input.len() as u64;
    let low = merge_accumulators(
        &acc,
        len.wrapping_mul(PRIME64_1),
        secret,
        SECRET_MERGE_ACC_START,
    );
    let high = merge_accumulators(
        &acc,
        !len.wrapping_mul(PRIME64_2),
        secret,
        secret.len() - STRIPE_SIZE - SECRET_MERGE_ACC_START,
    );
    make_u128(low, high)
}

#[inline]
fn accumulate_long<K: Xxh3Kernel>(kernel: K, input: &[u8], secret: &[u8]) -> [u64; 8] {
    debug_assert!(input.len() > MIDSIZE_MAX);
    debug_assert!(secret.len() >= SECRET_SIZE_MIN);
    let stripes_per_block = (secret.len() - STRIPE_SIZE) / SECRET_CONSUME_RATE;
    let mut accumulator = Accumulator::new(stripes_per_block);

    // Excluding the final byte leaves the last full or overlapping stripe for
    // the dedicated secret suffix below. `Accumulator` handles block scrambles,
    // so no potentially overflowing secret-derived block size is required.
    for stripe in input[..input.len() - 1].chunks_exact(STRIPE_SIZE) {
        accumulator.process(kernel, array_64(stripe, 0), secret);
    }

    let last_secret_offset = secret.len() - STRIPE_SIZE - SECRET_LAST_ACC_START;
    kernel.accumulate(
        &mut accumulator.lanes,
        array_64(input, input.len() - STRIPE_SIZE),
        array_64(secret, last_secret_offset),
    );
    accumulator.lanes
}

#[inline(always)]
fn merge_accumulators(acc: &[u64; 8], start: u64, secret: &[u8], secret_offset: usize) -> u64 {
    let mut result = start;
    for pair in 0..4 {
        let offset = secret_offset + pair * 16;
        result = result.wrapping_add(mul128_fold64(
            acc[pair * 2] ^ read_u64(secret, offset),
            acc[pair * 2 + 1] ^ read_u64(secret, offset + 8),
        ));
    }
    avalanche(result)
}

#[inline(always)]
fn array_64(input: &[u8], offset: usize) -> &[u8; 64] {
    input[offset..offset + 64]
        .try_into()
        .expect("validated XXH3 stripe range")
}

#[inline(always)]
unsafe fn secret_array_64(secret: &[u8], offset: usize) -> &[u8; 64] {
    debug_assert!(offset + STRIPE_SIZE <= secret.len());
    // SAFETY: The caller guarantees a complete 64-byte range at `offset`.
    // `[u8; 64]` has byte alignment, so the cast cannot be misaligned.
    unsafe { &*secret.as_ptr().add(offset).cast::<[u8; STRIPE_SIZE]>() }
}

#[derive(Clone, Copy)]
struct Accumulator {
    lanes: [u64; 8],
    stripe: usize,
    stripes_per_block: usize,
}

impl Accumulator {
    const fn new(stripes_per_block: usize) -> Self {
        Self {
            lanes: INITIAL_ACC,
            stripe: 0,
            stripes_per_block,
        }
    }

    #[inline(always)]
    fn process<K: Xxh3Kernel>(&mut self, kernel: K, stripe: &[u8; STRIPE_SIZE], secret: &[u8]) {
        let secret_offset = self.stripe * SECRET_CONSUME_RATE;
        // SAFETY: `stripe` is reset at `stripes_per_block`, which was derived
        // from the validated secret length when the accumulator was created.
        let secret_stripe = unsafe { secret_array_64(secret, secret_offset) };
        kernel.accumulate(&mut self.lanes, stripe, secret_stripe);
        self.stripe += 1;
        if self.stripe == self.stripes_per_block {
            // SAFETY: Every validated secret contains a complete 64-byte
            // suffix, and this offset selects that exact suffix.
            let scramble_secret = unsafe { secret_array_64(secret, secret.len() - STRIPE_SIZE) };
            kernel.scramble(&mut self.lanes, scramble_secret);
            self.stripe = 0;
        }
    }
}

#[derive(Clone)]
struct StreamState<S> {
    seed: u64,
    secret: S,
    use_custom_secret_for_short: bool,
    buffer: [u8; STREAM_BUFFER_SIZE],
    buffered: usize,
    accumulator: Accumulator,
    total_len: u64,
    length_overflowed: bool,
}

impl StreamState<[u8; DEFAULT_SECRET_SIZE]> {
    fn with_seed(seed: u64) -> Self {
        let stripes_per_block = (DEFAULT_SECRET_SIZE - STRIPE_SIZE) / SECRET_CONSUME_RATE;
        Self {
            seed,
            secret: derive_secret(seed),
            use_custom_secret_for_short: false,
            buffer: [0; STREAM_BUFFER_SIZE],
            buffered: 0,
            accumulator: Accumulator::new(stripes_per_block),
            total_len: 0,
            length_overflowed: false,
        }
    }
}

impl<'a> StreamState<&'a [u8]> {
    fn with_secret(seed: u64, secret: &'a [u8], use_custom_secret_for_short: bool) -> Self {
        let stripes_per_block = (secret.len() - STRIPE_SIZE) / SECRET_CONSUME_RATE;
        Self {
            seed,
            secret,
            use_custom_secret_for_short,
            buffer: [0; STREAM_BUFFER_SIZE],
            buffered: 0,
            accumulator: Accumulator::new(stripes_per_block),
            total_len: 0,
            length_overflowed: false,
        }
    }
}

impl<S: AsRef<[u8]>> StreamState<S> {
    fn reset(&mut self) {
        self.buffered = 0;
        self.accumulator = Accumulator::new(self.accumulator.stripes_per_block);
        self.total_len = 0;
        self.length_overflowed = false;
    }

    #[inline]
    fn update(&mut self, input: &[u8]) {
        if input.is_empty() {
            return;
        }

        let (total_len, overflowed) = self.total_len.overflowing_add(input.len() as u64);
        self.total_len = total_len;
        self.length_overflowed |= overflowed;
        let available = STREAM_BUFFER_SIZE - self.buffered;
        if input.len() <= available {
            self.buffer[self.buffered..self.buffered + input.len()].copy_from_slice(input);
            self.buffered += input.len();
            return;
        }

        kernel::dispatch!(update_stream(self, input));
    }

    #[inline]
    fn digest_64(&self) -> u64 {
        if !self.length_overflowed && self.total_len <= MIDSIZE_MAX as u64 {
            let (seed, secret) = if self.use_custom_secret_for_short {
                (0, self.secret.as_ref())
            } else {
                (self.seed, DEFAULT_SECRET.as_slice())
            };
            hash_64_short(&self.buffer[..self.total_len as usize], seed, secret)
        } else {
            kernel::dispatch!(finalize_stream_64(self))
        }
    }

    #[inline]
    fn digest_128(&self) -> u128 {
        if !self.length_overflowed && self.total_len <= MIDSIZE_MAX as u64 {
            let (seed, secret) = if self.use_custom_secret_for_short {
                (0, self.secret.as_ref())
            } else {
                (self.seed, DEFAULT_SECRET.as_slice())
            };
            hash_128_short(&self.buffer[..self.total_len as usize], seed, secret)
        } else {
            kernel::dispatch!(finalize_stream_128(self))
        }
    }

    const fn reported_total_len(&self) -> u64 {
        if self.length_overflowed {
            u64::MAX
        } else {
            self.total_len
        }
    }
}

#[inline]
fn update_stream<K: Xxh3Kernel, S: AsRef<[u8]>>(
    kernel: K,
    state: &mut StreamState<S>,
    mut input: &[u8],
) {
    let secret = state.secret.as_ref();
    let buffer = &mut state.buffer;
    let buffered = &mut state.buffered;
    let mut accumulator = state.accumulator;

    if *buffered != 0 {
        let available = STREAM_BUFFER_SIZE - *buffered;
        let copied = available.min(input.len());
        buffer[*buffered..*buffered + copied].copy_from_slice(&input[..copied]);
        *buffered += copied;
        input = &input[copied..];

        if *buffered < STREAM_BUFFER_SIZE || input.is_empty() {
            return;
        }

        for stripe in 0..(STREAM_BUFFER_SIZE / STRIPE_SIZE) {
            accumulator.process(kernel, array_64(buffer, stripe * STRIPE_SIZE), secret);
        }
        *buffered = 0;
    }

    if input.len() > STRIPE_SIZE {
        let process_len = ((input.len() - STRIPE_SIZE) / STRIPE_SIZE) * STRIPE_SIZE;
        for offset in (0..process_len).step_by(STRIPE_SIZE) {
            accumulator.process(kernel, array_64(input, offset), secret);
        }
        input = &input[process_len..];
    }

    buffer[..input.len()].copy_from_slice(input);
    *buffered = input.len();
    state.accumulator = accumulator;
}

#[inline]
fn finalize_stream_acc<K: Xxh3Kernel, S: AsRef<[u8]>>(
    kernel: K,
    state: &StreamState<S>,
) -> [u64; 8] {
    let mut accumulator = state.accumulator;
    let input = &state.buffer[..state.buffered];
    let full_stripes = input.len() / STRIPE_SIZE;
    let regular_stripes = if !input.is_empty() && input.len() % STRIPE_SIZE == 0 {
        full_stripes - 1
    } else {
        full_stripes
    };
    for stripe in 0..regular_stripes {
        accumulator.process(
            kernel,
            array_64(input, stripe * STRIPE_SIZE),
            state.secret.as_ref(),
        );
    }

    let mut temporary = [0_u8; STRIPE_SIZE];
    let last_stripe = if input.len() >= STRIPE_SIZE {
        array_64(input, input.len() - STRIPE_SIZE)
    } else {
        let reused = STRIPE_SIZE - input.len();
        temporary[..reused]
            .copy_from_slice(&state.buffer[STREAM_BUFFER_SIZE - reused..STREAM_BUFFER_SIZE]);
        temporary[reused..].copy_from_slice(input);
        &temporary
    };

    let secret = state.secret.as_ref();
    let last_secret_offset = secret.len() - STRIPE_SIZE - SECRET_LAST_ACC_START;
    kernel.accumulate(
        &mut accumulator.lanes,
        last_stripe,
        array_64(secret, last_secret_offset),
    );
    accumulator.lanes
}

#[inline]
fn finalize_stream_64<K: Xxh3Kernel, S: AsRef<[u8]>>(kernel: K, state: &StreamState<S>) -> u64 {
    let acc = finalize_stream_acc(kernel, state);
    merge_accumulators(
        &acc,
        state.total_len.wrapping_mul(PRIME64_1),
        state.secret.as_ref(),
        SECRET_MERGE_ACC_START,
    )
}

#[inline]
fn finalize_stream_128<K: Xxh3Kernel, S: AsRef<[u8]>>(kernel: K, state: &StreamState<S>) -> u128 {
    let acc = finalize_stream_acc(kernel, state);
    let low = merge_accumulators(
        &acc,
        state.total_len.wrapping_mul(PRIME64_1),
        state.secret.as_ref(),
        SECRET_MERGE_ACC_START,
    );
    let high = merge_accumulators(
        &acc,
        !state.total_len.wrapping_mul(PRIME64_2),
        state.secret.as_ref(),
        state.secret.as_ref().len() - STRIPE_SIZE - SECRET_MERGE_ACC_START,
    );
    make_u128(low, high)
}

/// Incremental XXH3-64 state.
///
/// The default and seeded forms own a 192-byte secret. Custom-secret forms
/// borrow the caller's secret. Every form owns fixed-size working buffers and
/// performs no allocation.
#[derive(Clone)]
pub struct Xxh3<S = [u8; DEFAULT_SECRET_SIZE]>(StreamState<S>);

impl Xxh3<[u8; DEFAULT_SECRET_SIZE]> {
    /// Creates an unseeded XXH3-64 state.
    #[must_use]
    pub fn new() -> Self {
        Self::with_seed(0)
    }

    /// Creates an XXH3-64 state with `seed`.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self(StreamState::with_seed(seed))
    }

    /// Hashes a complete byte slice without constructing streaming state.
    #[must_use]
    #[inline]
    pub fn oneshot(input: &[u8]) -> u64 {
        xxh3_64(input)
    }

    /// Hashes a complete byte slice with `seed`.
    #[must_use]
    #[inline]
    pub fn oneshot_with_seed(input: &[u8], seed: u64) -> u64 {
        xxh3_64_with_seed(input, seed)
    }

    /// Hashes a complete byte slice with a custom secret.
    #[inline]
    pub fn oneshot_with_secret(input: &[u8], secret: &[u8]) -> Result<u64, Xxh3SecretTooShort> {
        xxh3_64_with_secret(input, secret)
    }

    /// Hashes a complete byte slice with a seed and custom secret.
    #[inline]
    pub fn oneshot_with_seed_and_secret(
        input: &[u8],
        seed: u64,
        secret: &[u8],
    ) -> Result<u64, Xxh3SecretTooShort> {
        xxh3_64_with_seed_and_secret(input, seed, secret)
    }
}

impl<'a> Xxh3<&'a [u8]> {
    /// Creates an XXH3-64 state borrowing `secret`.
    pub fn with_secret(secret: &'a [u8]) -> Result<Self, Xxh3SecretTooShort> {
        validate_secret(secret)?;
        Ok(Self(StreamState::with_secret(0, secret, true)))
    }

    /// Creates an XXH3-64 state borrowing `secret` and using `seed` for short inputs.
    ///
    /// Inputs up to 240 bytes use `seed`, while longer inputs use `secret`.
    pub fn with_seed_and_secret(seed: u64, secret: &'a [u8]) -> Result<Self, Xxh3SecretTooShort> {
        validate_secret(secret)?;
        Ok(Self(StreamState::with_secret(seed, secret, false)))
    }
}

impl<S: AsRef<[u8]>> Xxh3<S> {
    /// Adds raw bytes to the hash state.
    #[inline]
    pub fn update(&mut self, input: &[u8]) {
        self.0.update(input);
    }

    /// Returns the digest without consuming the state.
    #[must_use]
    #[inline]
    pub fn digest(&self) -> u64 {
        self.0.digest_64()
    }

    /// Resets the state while retaining its seed and secret.
    pub fn reset(&mut self) {
        self.0.reset();
    }

    /// Returns the seed used by this state.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.0.seed
    }

    /// Returns the number of bytes written so far, saturating at [`u64::MAX`].
    #[must_use]
    pub const fn total_len(&self) -> u64 {
        self.0.reported_total_len()
    }
}

impl Default for Xxh3<[u8; DEFAULT_SECRET_SIZE]> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: AsRef<[u8]>> fmt::Debug for Xxh3<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Xxh3")
            .field("seed", &self.seed())
            .field("total_len", &self.total_len())
            .finish_non_exhaustive()
    }
}

impl<S: AsRef<[u8]>> Hasher for Xxh3<S> {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.digest()
    }
}

/// Explicitly named alias for the XXH3-64 streaming state.
pub type Xxh3_64<S = [u8; DEFAULT_SECRET_SIZE]> = Xxh3<S>;

/// Incremental XXH3-128 state.
///
/// This type has an inherent [`write`](Self::write) method rather than a
/// [`Hasher`] implementation because that trait only returns 64-bit digests.
#[derive(Clone)]
pub struct Xxh3_128<S = [u8; DEFAULT_SECRET_SIZE]>(StreamState<S>);

impl Xxh3_128<[u8; DEFAULT_SECRET_SIZE]> {
    /// Creates an unseeded XXH3-128 state.
    #[must_use]
    pub fn new() -> Self {
        Self::with_seed(0)
    }

    /// Creates an XXH3-128 state with `seed`.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self(StreamState::with_seed(seed))
    }

    /// Hashes a complete byte slice without constructing streaming state.
    #[must_use]
    #[inline]
    pub fn oneshot(input: &[u8]) -> u128 {
        xxh3_128(input)
    }

    /// Hashes a complete byte slice with `seed`.
    #[must_use]
    #[inline]
    pub fn oneshot_with_seed(input: &[u8], seed: u64) -> u128 {
        xxh3_128_with_seed(input, seed)
    }

    /// Hashes a complete byte slice with a custom secret.
    #[inline]
    pub fn oneshot_with_secret(input: &[u8], secret: &[u8]) -> Result<u128, Xxh3SecretTooShort> {
        xxh3_128_with_secret(input, secret)
    }

    /// Hashes a complete byte slice with a seed and custom secret.
    #[inline]
    pub fn oneshot_with_seed_and_secret(
        input: &[u8],
        seed: u64,
        secret: &[u8],
    ) -> Result<u128, Xxh3SecretTooShort> {
        xxh3_128_with_seed_and_secret(input, seed, secret)
    }
}

impl<'a> Xxh3_128<&'a [u8]> {
    /// Creates an XXH3-128 state borrowing `secret`.
    pub fn with_secret(secret: &'a [u8]) -> Result<Self, Xxh3SecretTooShort> {
        validate_secret(secret)?;
        Ok(Self(StreamState::with_secret(0, secret, true)))
    }

    /// Creates an XXH3-128 state borrowing `secret` and using `seed` for short inputs.
    ///
    /// Inputs up to 240 bytes use `seed`, while longer inputs use `secret`.
    pub fn with_seed_and_secret(seed: u64, secret: &'a [u8]) -> Result<Self, Xxh3SecretTooShort> {
        validate_secret(secret)?;
        Ok(Self(StreamState::with_secret(seed, secret, false)))
    }
}

impl<S: AsRef<[u8]>> Xxh3_128<S> {
    /// Adds raw bytes to the hash state.
    #[inline]
    pub fn write(&mut self, input: &[u8]) {
        self.0.update(input);
    }

    /// Alias for [`write`](Self::write), matching the other streaming types.
    #[inline]
    pub fn update(&mut self, input: &[u8]) {
        self.write(input);
    }

    /// Returns the 128-bit digest without consuming the state.
    #[must_use]
    #[inline]
    pub fn digest(&self) -> u128 {
        self.0.digest_128()
    }

    /// Alias for [`digest`](Self::digest), matching common XXH3 APIs.
    #[must_use]
    #[inline]
    pub fn finish_128(&self) -> u128 {
        self.digest()
    }

    /// Resets the state while retaining its seed and secret.
    pub fn reset(&mut self) {
        self.0.reset();
    }

    /// Returns the seed used by this state.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.0.seed
    }

    /// Returns the number of bytes written so far, saturating at [`u64::MAX`].
    #[must_use]
    pub const fn total_len(&self) -> u64 {
        self.0.reported_total_len()
    }
}

impl Default for Xxh3_128<[u8; DEFAULT_SECRET_SIZE]> {
    fn default() -> Self {
        Self::new()
    }
}

impl<S: AsRef<[u8]>> fmt::Debug for Xxh3_128<S> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Xxh3_128")
            .field("seed", &self.seed())
            .field("total_len", &self.total_len())
            .finish_non_exhaustive()
    }
}

/// Deterministic [`BuildHasher`] for [`Xxh3`].
///
/// This builder is intended for trusted inputs. It does not randomize its seed
/// and is not resistant to deliberate hash-flooding attacks.
#[derive(Clone, Copy, Debug, Default)]
pub struct Xxh3Builder {
    seed: u64,
}

impl Xxh3Builder {
    /// Creates a builder using `seed`.
    #[must_use]
    pub const fn with_seed(seed: u64) -> Self {
        Self { seed }
    }
}

impl BuildHasher for Xxh3Builder {
    type Hasher = Xxh3;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Xxh3::with_seed(self.seed)
    }
}

/// Deterministic [`BuildHasher`] for an XXH3 custom secret.
///
/// The builder borrows the secret and never allocates. It is intended for
/// trusted inputs and does not make XXH3 resistant to deliberate hash flooding.
#[derive(Clone, Copy)]
pub struct Xxh3SecretBuilder<'a> {
    seed: u64,
    secret: &'a [u8],
    use_custom_secret_for_short: bool,
}

impl<'a> Xxh3SecretBuilder<'a> {
    /// Creates a builder using `secret` for inputs of every length.
    pub fn with_secret(secret: &'a [u8]) -> Result<Self, Xxh3SecretTooShort> {
        validate_secret(secret)?;
        Ok(Self {
            seed: 0,
            secret,
            use_custom_secret_for_short: true,
        })
    }

    /// Creates a builder using `seed` for short inputs and `secret` for long inputs.
    pub fn with_seed_and_secret(seed: u64, secret: &'a [u8]) -> Result<Self, Xxh3SecretTooShort> {
        validate_secret(secret)?;
        Ok(Self {
            seed,
            secret,
            use_custom_secret_for_short: false,
        })
    }
}

impl fmt::Debug for Xxh3SecretBuilder<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Xxh3SecretBuilder")
            .field("seed", &self.seed)
            .field("secret_len", &self.secret.len())
            .finish_non_exhaustive()
    }
}

impl<'a> BuildHasher for Xxh3SecretBuilder<'a> {
    type Hasher = Xxh3<&'a [u8]>;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Xxh3(StreamState::with_secret(
            self.seed,
            self.secret,
            self.use_custom_secret_for_short,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_kernel_matches_scalar<K: Xxh3Kernel>(kernel: K) {
        for len in [241_usize, 256, 1_023, 1_024, 1_025, 4_097] {
            let input: std::vec::Vec<_> = (0..len)
                .map(|index| index.wrapping_mul(131).wrapping_add(17) as u8)
                .collect();
            for seed in [0, 1, 0x0123_4567_89ab_cdef] {
                let secret = derive_secret(seed);
                assert_eq!(
                    hash_long_64(kernel, &input, &secret),
                    hash_long_64(crate::xxhash::kernel::Scalar, &input, &secret),
                    "XXH3-64 length={len} seed={seed:#x}"
                );
                assert_eq!(
                    hash_long_128(kernel, &input, &secret),
                    hash_long_128(crate::xxhash::kernel::Scalar, &input, &secret),
                    "XXH3-128 length={len} seed={seed:#x}"
                );
            }

            for secret_len in [SECRET_SIZE_MIN, DEFAULT_SECRET_SIZE, 255, 1_024] {
                let secret: std::vec::Vec<_> = (0..secret_len)
                    .map(|index| index.wrapping_mul(197).wrapping_add(0xa5) as u8)
                    .collect();
                assert_eq!(
                    hash_long_64(kernel, &input, &secret),
                    hash_long_64(crate::xxhash::kernel::Scalar, &input, &secret),
                    "XXH3-64 length={len} secret_len={secret_len}"
                );
                assert_eq!(
                    hash_long_128(kernel, &input, &secret),
                    hash_long_128(crate::xxhash::kernel::Scalar, &input, &secret),
                    "XXH3-128 length={len} secret_len={secret_len}"
                );
            }
        }
    }

    #[test]
    fn official_empty_vectors() {
        assert_eq!(xxh3_64(b""), 0x2d06_8005_38d3_94c2);
        assert_eq!(xxh3_128(b""), 0x99aa_06d3_0147_98d8_6001_c324_468d_497f);
    }

    #[test]
    fn reset_reuses_state() {
        let mut hash = Xxh3::with_seed(42);
        hash.update(b"before");
        hash.reset();
        hash.update(b"after");
        assert_eq!(hash.digest(), xxh3_64_with_seed(b"after", 42));
    }

    #[test]
    fn length_overflow_keeps_long_digest_mode() {
        let mut hash = Xxh3::new();
        hash.0.total_len = u64::MAX;
        hash.update(&[0]);
        assert!(hash.0.length_overflowed);
        assert_eq!(hash.total_len(), u64::MAX);
        assert_eq!(
            hash.digest(),
            kernel::dispatch!(finalize_stream_64(&hash.0))
        );

        let mut hash = Xxh3_128::new();
        hash.0.total_len = u64::MAX;
        hash.update(&[0]);
        assert!(hash.0.length_overflowed);
        assert_eq!(hash.total_len(), u64::MAX);
        assert_eq!(
            hash.digest(),
            kernel::dispatch!(finalize_stream_128(&hash.0))
        );
    }

    #[test]
    fn scalar_kernel_matches_selected_backend() {
        for len in [241_usize, 256, 1_023, 1_024, 1_025, 4_097] {
            let input: std::vec::Vec<_> = (0..len)
                .map(|index| index.wrapping_mul(131).wrapping_add(17) as u8)
                .collect();
            for seed in [0, 1, 0x0123_4567_89ab_cdef] {
                let secret = derive_secret(seed);
                assert_eq!(
                    hash_long_64(crate::xxhash::kernel::Scalar, &input, &secret),
                    xxh3_64_with_seed(&input, seed)
                );
                assert_eq!(
                    hash_long_128(crate::xxhash::kernel::Scalar, &input, &secret),
                    xxh3_128_with_seed(&input, seed)
                );
            }
        }
    }

    #[test]
    fn every_available_hardware_kernel_matches_scalar() {
        #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
        if crate::xxhash::kernel::Backend::Neon.is_available() {
            // SAFETY: Availability was checked immediately above.
            assert_kernel_matches_scalar(unsafe { crate::xxhash::kernel::Neon::new_unchecked() });
        }

        #[cfg(target_arch = "x86_64")]
        {
            if crate::xxhash::kernel::Backend::Sse2.is_available() {
                // SAFETY: Availability was checked immediately above.
                assert_kernel_matches_scalar(unsafe {
                    crate::xxhash::kernel::Sse2::new_unchecked()
                });
            }
            if crate::xxhash::kernel::Backend::Avx2.is_available() {
                // SAFETY: Availability was checked immediately above.
                assert_kernel_matches_scalar(unsafe {
                    crate::xxhash::kernel::Avx2::new_unchecked()
                });
            }
        }
    }
}
