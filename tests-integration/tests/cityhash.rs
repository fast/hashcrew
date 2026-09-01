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

use rache::cityhash::cityhash32;
use rache::cityhash::cityhash64;
use rache::cityhash::cityhash64_with_seed;
use rache::cityhash::cityhash64_with_seeds;
use rache::cityhash::cityhash128;
use rache::cityhash::cityhash128_to_64;
use rache::cityhash::cityhash128_with_seed;

mod support;

use support::next_random;
use support::random_input;

const LENGTHS: &[usize] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 23, 24, 25, 31, 32, 33, 63, 64,
    65, 95, 96, 97, 127, 128, 129, 159, 160, 191, 192, 223, 239, 240, 241, 255, 256, 257, 511, 512,
    513, 1_023, 1_024, 1_025, 4_097,
];
const SEEDS: &[u64] = &[0, 1, 0x0123_4567_89ab_cdef, u64::MAX];
const K0: u64 = 0xc3a5_c85c_97cb_3127;

fn input(len: usize) -> Vec<u8> {
    (0..len)
        .map(|index| index.wrapping_mul(131).wrapping_add(17) as u8)
        .collect()
}

fn reference_128(input: &[u8]) -> u128 {
    cityhash_rs::cityhash_110_128(input).rotate_left(64)
}

fn read_u64(input: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes(input[offset..offset + 8].try_into().unwrap())
}

#[test]
fn oneshot_matches_independent_implementations() {
    for &len in LENGTHS {
        let bytes = input(len);
        assert_eq!(
            cityhash32(&bytes),
            cityhasher::hash::<u32>(&bytes),
            "CityHash32 length={len}"
        );
        assert_eq!(
            cityhash64(&bytes),
            cityhasher::hash::<u64>(&bytes),
            "CityHash64 length={len}"
        );
        assert_eq!(
            cityhash128(&bytes),
            reference_128(&bytes),
            "CityHash128 length={len}"
        );
    }
}

#[test]
fn every_length_through_two_kib_matches_independent_implementations() {
    let bytes = input(2 * 1_024);
    for len in 0..=bytes.len() {
        let input = &bytes[..len];
        let seed = (len as u64)
            .wrapping_mul(0x9e37_79b1_85eb_ca87)
            .rotate_left((len % 64) as u32);

        assert_eq!(
            cityhash32(input),
            cityhasher::hash::<u32>(input),
            "CityHash32 length={len}"
        );
        assert_eq!(
            cityhash64(input),
            cityhasher::hash::<u64>(input),
            "CityHash64 length={len}"
        );
        assert_eq!(
            cityhash64_with_seed(input, seed),
            cityhasher::hash_with_seed::<u64>(input, seed),
            "CityHash64WithSeed length={len} seed={seed:#x}"
        );
        assert_eq!(
            cityhash128(input),
            reference_128(input),
            "CityHash128 length={len}"
        );
    }
}

#[test]
fn seeded_64_matches_independent_implementation() {
    for &len in LENGTHS {
        let bytes = input(len);
        for &seed in SEEDS {
            assert_eq!(
                cityhash64_with_seed(&bytes, seed),
                cityhasher::hash_with_seed::<u64>(&bytes, seed),
                "CityHash64WithSeed length={len} seed={seed:#x}"
            );
        }
    }
}

#[test]
fn randomized_inputs_match_independent_implementations() {
    let mut random = 0x6a09_e667_f3bc_c909;
    for case in 0..256 {
        let len = next_random(&mut random) as usize % (128 * 1_024);
        let seed = next_random(&mut random);
        let bytes = random_input(&mut random, len);

        assert_eq!(
            cityhash32(&bytes),
            cityhasher::hash::<u32>(&bytes),
            "CityHash32 case={case} length={len}"
        );
        assert_eq!(
            cityhash64(&bytes),
            cityhasher::hash::<u64>(&bytes),
            "CityHash64 case={case} length={len}"
        );
        assert_eq!(
            cityhash64_with_seed(&bytes, seed),
            cityhasher::hash_with_seed::<u64>(&bytes, seed),
            "CityHash64WithSeed case={case} length={len}"
        );
        assert_eq!(
            cityhash128(&bytes),
            reference_128(&bytes),
            "CityHash128 case={case} length={len}"
        );
    }
}

#[test]
fn seeded_apis_preserve_official_word_order_and_relationships() {
    let bytes = input(4_097);
    let seed0 = 0x0123_4567_89ab_cdef;
    let seed1 = 0xfedc_ba98_7654_3210;
    let reduced_input =
        (u128::from(seed1) << 64) | u128::from(cityhash64(&bytes).wrapping_sub(seed0));
    assert_eq!(
        cityhash64_with_seeds(&bytes, seed0, seed1),
        cityhash128_to_64(reduced_input)
    );

    let low = read_u64(&bytes, 0);
    let high = read_u64(&bytes, 8).wrapping_add(K0);
    let seed128 = (u128::from(high) << 64) | u128::from(low);
    assert_eq!(
        cityhash128(&bytes),
        cityhash128_with_seed(&bytes[16..], seed128)
    );
}
