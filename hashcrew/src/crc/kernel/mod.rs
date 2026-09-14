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

#[cfg(all(target_arch = "aarch64", target_endian = "little", not(miri)))]
mod aarch64;
#[cfg(all(target_arch = "x86_64", not(miri)))]
mod x86;

#[inline]
pub(super) fn update<const CASTAGNOLI: bool>(state: u32, input: &[u8]) -> u32 {
    if input.is_empty() {
        return state;
    }
    #[cfg(all(target_arch = "aarch64", target_endian = "little", not(miri)))]
    if aarch64::crc_available() {
        // SAFETY: The availability check covers the CRC instructions; the
        // architecture wrapper separately checks features needed for folding.
        return unsafe { aarch64::update::<CASTAGNOLI>(state, input) };
    }
    #[cfg(all(target_arch = "x86_64", not(miri)))]
    {
        x86::update::<CASTAGNOLI>(state, input)
    }
    #[cfg(not(all(target_arch = "x86_64", not(miri))))]
    {
        scalar::<CASTAGNOLI>(state, input)
    }
}

#[inline]
fn scalar<const CASTAGNOLI: bool>(state: u32, input: &[u8]) -> u32 {
    let table = if CASTAGNOLI {
        &super::ISCSI_TABLE
    } else {
        &super::ISO_HDLC_TABLE
    };
    super::scalar::update(state, input, table)
}

// Reflected representation of x^bits modulo the selected polynomial.
#[cfg(all(
    any(target_arch = "aarch64", target_arch = "x86_64"),
    target_endian = "little",
    not(miri)
))]
const fn power<const CASTAGNOLI: bool>(mut bits: usize) -> u32 {
    let polynomial = if CASTAGNOLI {
        super::ISCSI_POLYNOMIAL
    } else {
        super::ISO_HDLC_POLYNOMIAL
    };
    let mut value = 0x8000_0000;
    while bits != 0 {
        value = (value >> 1) ^ (polynomial & 0_u32.wrapping_sub(value & 1));
        bits -= 1;
    }
    value
}

#[cfg(all(
    any(target_arch = "aarch64", target_arch = "x86_64"),
    target_endian = "little",
    not(miri)
))]
const fn folding_factors<const CASTAGNOLI: bool>(bytes: usize) -> [u64; 2] {
    // The low and high halves of a folded vector are separated by 64 powers.
    [
        power::<CASTAGNOLI>(bytes * 8 + 31) as u64,
        power::<CASTAGNOLI>(bytes * 8 - 33) as u64,
    ]
}

#[cfg(all(
    test,
    any(target_arch = "aarch64", target_arch = "x86_64"),
    target_endian = "little",
    not(miri)
))]
unsafe fn test_backend<const CASTAGNOLI: bool>(backend: unsafe fn(u32, &[u8]) -> u32) {
    let mut bytes = std::vec![0; 16_384 + 64];
    let mut random = 0x4f1b_bcdd_970e_4169_u64;
    for byte in &mut bytes {
        random ^= random << 13;
        random ^= random >> 7;
        random ^= random << 17;
        *byte = random as u8;
    }
    for offset in 0..64 {
        for len in (0..=256).chain([
            383, 384, 385, 511, 512, 513, 767, 768, 769, 1023, 1024, 1025, 4095, 4096, 4097, 8191,
            8192, 8193, 16_384,
        ]) {
            let input = &bytes[offset..offset + len];
            for state in [0, u32::MAX, 0x739a_504d] {
                let expected = scalar::<CASTAGNOLI>(state, input);
                // SAFETY: The test caller verifies the backend's CPU features.
                let actual = unsafe { backend(state, input) };
                assert_eq!(
                    actual, expected,
                    "offset={offset} len={len} state={state:08x}"
                );
            }
        }
    }
}
