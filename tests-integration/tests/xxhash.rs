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

use core::hash::{BuildHasher, Hasher};
use std::collections::HashMap;

use rache::{
    SECRET_SIZE_MIN, Xxh3, Xxh3_128, Xxh3Builder, Xxh3SecretBuilder, Xxh32, Xxh32Builder, Xxh64,
    Xxh64Builder, xxh3_64, xxh3_64_with_secret, xxh3_64_with_seed, xxh3_64_with_seed_and_secret,
    xxh3_128, xxh3_128_with_secret, xxh3_128_with_seed, xxh3_128_with_seed_and_secret, xxh32,
    xxh64,
};
use xxhash_rust::{xxh3, xxh32 as reference32, xxh64 as reference64};

mod support;

use support::{next_random, random_input};

const LENGTHS: &[usize] = &[
    0, 1, 2, 3, 4, 7, 8, 9, 15, 16, 17, 31, 32, 33, 63, 64, 65, 95, 96, 97, 127, 128, 129, 159,
    160, 191, 192, 223, 239, 240, 241, 255, 256, 257, 511, 512, 513, 1_023, 1_024, 1_025, 2_047,
    2_048, 4_097, 16_384,
];
const SEEDS: &[u64] = &[0, 1, 0x0123_4567_89ab_cdef, u64::MAX];

fn input(len: usize) -> Vec<u8> {
    (0..len)
        .map(|index| {
            let value = index
                .wrapping_mul(0x9e37)
                .wrapping_add(index.rotate_left(7))
                .wrapping_add(0x5a);
            value as u8
        })
        .collect()
}

fn secret(len: usize) -> Vec<u8> {
    (0..len)
        .map(|index| index.wrapping_mul(197).wrapping_add(0xa5) as u8)
        .collect()
}

#[test]
fn official_empty_vectors_are_stable() {
    assert_eq!(xxh32(b"", 0), 0x02cc_5d05);
    assert_eq!(xxh64(b"", 0), 0xef46_db37_51d8_e999);
    assert_eq!(xxh3_64(b""), 0x2d06_8005_38d3_94c2);
    assert_eq!(xxh3_128(b""), 0x99aa_06d3_0147_98d8_6001_c324_468d_497f);
}

#[test]
fn oneshot_matches_reference() {
    for &len in LENGTHS {
        let bytes = input(len);
        for &seed in SEEDS {
            assert_eq!(
                xxh32(&bytes, seed as u32),
                reference32::xxh32(&bytes, seed as u32),
                "XXH32 length={len} seed={seed:#x}"
            );
            assert_eq!(
                xxh64(&bytes, seed),
                reference64::xxh64(&bytes, seed),
                "XXH64 length={len} seed={seed:#x}"
            );
            assert_eq!(
                xxh3_64_with_seed(&bytes, seed),
                xxh3::xxh3_64_with_seed(&bytes, seed),
                "XXH3-64 length={len} seed={seed:#x}"
            );
            assert_eq!(
                xxh3_128_with_seed(&bytes, seed),
                xxh3::xxh3_128_with_seed(&bytes, seed),
                "XXH3-128 length={len} seed={seed:#x}"
            );
        }
    }
}

#[test]
fn custom_secret_oneshot_matches_independent_implementations() {
    for secret_len in [SECRET_SIZE_MIN, SECRET_SIZE_MIN + 1, 192, 255, 1_024] {
        let secret = secret(secret_len);
        for &len in LENGTHS {
            let bytes = input(len);
            assert_eq!(
                xxh3_64_with_secret(&bytes, &secret).unwrap(),
                xxh3::xxh3_64_with_secret(&bytes, &secret),
                "XXH3-64 length={len} secret_len={secret_len}"
            );
            assert_eq!(
                xxh3_128_with_secret(&bytes, &secret).unwrap(),
                xxh3::xxh3_128_with_secret(&bytes, &secret),
                "XXH3-128 length={len} secret_len={secret_len}"
            );
        }
    }
}

#[test]
fn custom_secret_matches_reference_for_every_length_through_variable_blocks() {
    let bytes = input(2 * 1_024 + 1);
    for secret_len in [SECRET_SIZE_MIN, 192, 255, 1_024] {
        let secret = secret(secret_len);
        for len in 0..bytes.len() {
            assert_eq!(
                xxh3_64_with_secret(&bytes[..len], &secret).unwrap(),
                xxh3::xxh3_64_with_secret(&bytes[..len], &secret),
                "XXH3-64 length={len} secret_len={secret_len}"
            );
            assert_eq!(
                xxh3_128_with_secret(&bytes[..len], &secret).unwrap(),
                xxh3::xxh3_128_with_secret(&bytes[..len], &secret),
                "XXH3-128 length={len} secret_len={secret_len}"
            );
        }
    }
}

#[test]
fn seed_and_secret_oneshot_matches_reference_contract() {
    for secret_len in [SECRET_SIZE_MIN, 192, 255, 1_024] {
        let secret = secret(secret_len);
        for &len in LENGTHS {
            let bytes = input(len);
            for &seed in SEEDS {
                assert_eq!(
                    xxh3_64_with_seed_and_secret(&bytes, seed, &secret).unwrap(),
                    twox_hash::xxhash3_64::Hasher::oneshot_with_seed_and_secret(
                        seed, &secret, &bytes,
                    )
                    .unwrap(),
                    "XXH3-64 length={len} seed={seed:#x} secret_len={secret_len}"
                );
                assert_eq!(
                    xxh3_128_with_seed_and_secret(&bytes, seed, &secret).unwrap(),
                    twox_hash::xxhash3_128::Hasher::oneshot_with_seed_and_secret(
                        seed, &secret, &bytes,
                    )
                    .unwrap(),
                    "XXH3-128 length={len} seed={seed:#x} secret_len={secret_len}"
                );
            }
        }
    }
}

#[test]
fn custom_secret_length_is_validated() {
    let short = secret(SECRET_SIZE_MIN - 1);
    let error = xxh3_64_with_secret(b"rache", &short).unwrap_err();
    assert_eq!(error.actual_len(), SECRET_SIZE_MIN - 1);
    assert_eq!(rache::Xxh3SecretTooShort::minimum_len(), SECRET_SIZE_MIN);
    assert_eq!(
        error.to_string(),
        "XXH3 secret is 135 bytes; at least 136 bytes are required"
    );

    assert!(xxh3_128_with_secret(b"rache", &short).is_err());
    assert!(xxh3_64_with_seed_and_secret(b"rache", 7, &short).is_err());
    assert!(xxh3_128_with_seed_and_secret(b"rache", 7, &short).is_err());
    assert!(Xxh3::with_secret(&short).is_err());
    assert!(Xxh3::with_seed_and_secret(7, &short).is_err());
    assert!(Xxh3_128::with_secret(&short).is_err());
    assert!(Xxh3_128::with_seed_and_secret(7, &short).is_err());
    assert!(Xxh3SecretBuilder::with_secret(&short).is_err());
    assert!(Xxh3SecretBuilder::with_seed_and_secret(7, &short).is_err());
}

#[test]
fn custom_secret_states_and_builders_can_own_their_storage() {
    let bytes = input(4_097);
    let owned_secret = secret(255);
    let expected64 = xxh3_64_with_secret(&bytes, &owned_secret).unwrap();
    let mut hash64 = Xxh3::with_secret(owned_secret).unwrap();
    hash64.update(&bytes);
    assert_eq!(hash64.digest(), expected64);

    let secret = [0xa5; 192];
    let mut hash128 = Xxh3_128::with_seed_and_secret(7, secret).unwrap();
    hash128.update(&bytes);
    assert_eq!(
        hash128.digest(),
        xxh3_128_with_seed_and_secret(&bytes, 7, &secret).unwrap()
    );

    let builder = Xxh3SecretBuilder::with_secret(secret).unwrap();
    let mut map = HashMap::with_hasher(builder);
    map.insert("rache", 2);
    assert_eq!(map["rache"], 2);
}

#[test]
fn custom_secret_streaming_matches_oneshot() {
    let seed = 0x0123_4567_89ab_cdef;
    for secret_len in [SECRET_SIZE_MIN, 192, 255, 1_024] {
        let secret = secret(secret_len);
        for &len in LENGTHS {
            let bytes = input(len);
            for chunk_size in [1, 17, 64, 257] {
                let mut hash64 = Xxh3::with_secret(&secret).unwrap();
                let mut hash128 = Xxh3_128::with_secret(&secret).unwrap();
                let mut combined64 = Xxh3::with_seed_and_secret(seed, &secret).unwrap();
                let mut combined128 = Xxh3_128::with_seed_and_secret(seed, &secret).unwrap();

                for chunk in bytes.chunks(chunk_size) {
                    hash64.update(chunk);
                    hash128.update(chunk);
                    combined64.update(chunk);
                    combined128.update(chunk);
                }

                assert_eq!(
                    hash64.digest(),
                    xxh3_64_with_secret(&bytes, &secret).unwrap()
                );
                assert_eq!(
                    hash128.digest(),
                    xxh3_128_with_secret(&bytes, &secret).unwrap()
                );
                assert_eq!(
                    combined64.digest(),
                    xxh3_64_with_seed_and_secret(&bytes, seed, &secret).unwrap()
                );
                assert_eq!(
                    combined128.digest(),
                    xxh3_128_with_seed_and_secret(&bytes, seed, &secret).unwrap()
                );
            }
        }
    }
}

#[test]
fn xxh3_matches_reference_for_every_length_through_two_blocks() {
    let bytes = input(2 * 1_024 + 1);
    for len in 0..bytes.len() {
        let seed = (len as u64)
            .wrapping_mul(0x9e37_79b1_85eb_ca87)
            .rotate_left((len % 64) as u32);
        assert_eq!(
            xxh3_64_with_seed(&bytes[..len], seed),
            xxh3::xxh3_64_with_seed(&bytes[..len], seed),
            "XXH3-64 length={len} seed={seed:#x}"
        );
        assert_eq!(
            xxh3_128_with_seed(&bytes[..len], seed),
            xxh3::xxh3_128_with_seed(&bytes[..len], seed),
            "XXH3-128 length={len} seed={seed:#x}"
        );
    }
}

#[test]
fn randomized_inputs_match_reference() {
    let mut random = 0x6a09_e667_f3bc_c909;
    for case in 0..256 {
        let len = (next_random(&mut random) as usize) % (128 * 1_024);
        let seed = next_random(&mut random);
        let bytes = random_input(&mut random, len);

        assert_eq!(
            xxh32(&bytes, seed as u32),
            reference32::xxh32(&bytes, seed as u32),
            "XXH32 randomized case={case} length={len}"
        );
        assert_eq!(
            xxh64(&bytes, seed),
            reference64::xxh64(&bytes, seed),
            "XXH64 randomized case={case} length={len}"
        );
        assert_eq!(
            xxh3_64_with_seed(&bytes, seed),
            xxh3::xxh3_64_with_seed(&bytes, seed),
            "XXH3-64 randomized case={case} length={len}"
        );
        assert_eq!(
            xxh3_128_with_seed(&bytes, seed),
            xxh3::xxh3_128_with_seed(&bytes, seed),
            "XXH3-128 randomized case={case} length={len}"
        );
    }
}

#[test]
fn streaming_matches_oneshot_for_many_chunk_sizes() {
    let bytes = input(4_097);
    for &seed in SEEDS {
        for chunk_size in [1, 3, 15, 16, 31, 64, 65, 127, 256, 1_025] {
            let mut hash32 = Xxh32::with_seed(seed as u32);
            let mut hash64 = Xxh64::with_seed(seed);
            let mut hash3 = Xxh3::with_seed(seed);
            let mut hash128 = Xxh3_128::with_seed(seed);

            for chunk in bytes.chunks(chunk_size) {
                hash32.update(chunk);
                hash64.update(chunk);
                hash3.update(chunk);
                hash128.update(chunk);
            }

            assert_eq!(hash32.digest(), xxh32(&bytes, seed as u32));
            assert_eq!(hash64.digest(), xxh64(&bytes, seed));
            assert_eq!(hash3.digest(), xxh3_64_with_seed(&bytes, seed));
            assert_eq!(hash128.digest(), xxh3_128_with_seed(&bytes, seed));
        }
    }
}

#[test]
fn streaming_boundaries_match_oneshot() {
    for &len in LENGTHS {
        let bytes = input(len);
        let mut hash3 = Xxh3::with_seed(7);
        let mut hash128 = Xxh3_128::with_seed(7);
        for chunk in bytes.chunks(17) {
            hash3.update(chunk);
            hash128.update(chunk);
        }
        assert_eq!(hash3.digest(), xxh3_64_with_seed(&bytes, 7), "length={len}");
        assert_eq!(
            hash128.digest(),
            xxh3_128_with_seed(&bytes, 7),
            "length={len}"
        );
    }
}

#[test]
fn randomized_streaming_partitions_match_oneshot() {
    let mut random = 0xbb67_ae85_84ca_a73b;
    for case in 0..128 {
        let len = (next_random(&mut random) as usize) % (64 * 1_024);
        let seed = next_random(&mut random);
        let bytes = random_input(&mut random, len);
        let mut hash32 = Xxh32::with_seed(seed as u32);
        let mut hash64 = Xxh64::with_seed(seed);
        let mut hash3 = Xxh3::with_seed(seed);
        let mut hash128 = Xxh3_128::with_seed(seed);
        let mut offset = 0;

        while offset < bytes.len() {
            let chunk_len = 1 + (next_random(&mut random) as usize % 521);
            let end = (offset + chunk_len).min(bytes.len());
            let chunk = &bytes[offset..end];
            hash32.update(chunk);
            hash64.update(chunk);
            hash3.update(chunk);
            hash128.update(chunk);
            if next_random(&mut random) & 3 == 0 {
                hash32.update(&[]);
                hash64.update(&[]);
                hash3.update(&[]);
                hash128.update(&[]);
            }
            offset = end;
        }

        assert_eq!(hash32.digest(), xxh32(&bytes, seed as u32), "case={case}");
        assert_eq!(hash64.digest(), xxh64(&bytes, seed), "case={case}");
        assert_eq!(
            hash3.digest(),
            xxh3_64_with_seed(&bytes, seed),
            "case={case}"
        );
        assert_eq!(
            hash128.digest(),
            xxh3_128_with_seed(&bytes, seed),
            "case={case}"
        );
    }
}

#[test]
fn every_two_way_partition_matches_oneshot() {
    let bytes = input(257);
    for len in 0..=bytes.len() {
        let input = &bytes[..len];
        let seed = (len as u64)
            .wrapping_mul(0x9e37_79b1_85eb_ca87)
            .rotate_left((len % 64) as u32);
        let expected32 = xxh32(input, seed as u32);
        let expected64 = xxh64(input, seed);
        let expected3 = xxh3_64_with_seed(input, seed);
        let expected128 = xxh3_128_with_seed(input, seed);

        assert_eq!(Xxh32::oneshot(input, seed as u32), expected32);
        assert_eq!(Xxh64::oneshot(input, seed), expected64);
        assert_eq!(Xxh3::oneshot_with_seed(input, seed), expected3);
        assert_eq!(Xxh3_128::oneshot_with_seed(input, seed), expected128);

        for split in 0..=len {
            let mut hash32 = Xxh32::with_seed(seed as u32);
            let mut hash64 = Xxh64::with_seed(seed);
            let mut hash3 = Xxh3::with_seed(seed);
            let mut hash128 = Xxh3_128::with_seed(seed);

            for chunk in [&[][..], &input[..split], &[][..], &input[split..], &[][..]] {
                hash32.update(chunk);
                hash64.update(chunk);
                hash3.update(chunk);
                hash128.update(chunk);
            }

            assert_eq!(
                hash32.digest(),
                expected32,
                "XXH32 length={len} split={split}"
            );
            assert_eq!(
                hash64.digest(),
                expected64,
                "XXH64 length={len} split={split}"
            );
            assert_eq!(
                hash3.digest(),
                expected3,
                "XXH3-64 length={len} split={split}"
            );
            assert_eq!(
                hash128.digest(),
                expected128,
                "XXH3-128 length={len} split={split}"
            );
        }
    }
}

#[test]
fn streaming_state_can_be_finished_cloned_continued_and_reset() {
    let seed = 0x0123_4567_89ab_cdef;
    let prefix = input(1_337);
    let suffix = input(777);

    let mut hash3 = Xxh3::with_seed(seed);
    hash3.update(&prefix);
    assert_eq!(hash3.digest(), hash3.digest());
    assert_eq!(hash3.digest(), xxh3_64_with_seed(&prefix, seed));

    let mut fork = hash3.clone();
    hash3.update(&suffix);
    let joined: Vec<_> = prefix.iter().chain(&suffix).copied().collect();
    assert_eq!(hash3.digest(), xxh3_64_with_seed(&joined, seed));
    assert_eq!(fork.digest(), xxh3_64_with_seed(&prefix, seed));

    fork.reset();
    fork.update(&suffix);
    assert_eq!(fork.seed(), seed);
    assert_eq!(fork.total_len(), suffix.len() as u64);
    assert_eq!(fork.digest(), xxh3_64_with_seed(&suffix, seed));

    let mut hash128 = Xxh3_128::with_seed(seed);
    hash128.update(&prefix);
    let mut fork128 = hash128.clone();
    hash128.update(&suffix);
    assert_eq!(hash128.digest(), xxh3_128_with_seed(&joined, seed));
    assert_eq!(fork128.digest(), xxh3_128_with_seed(&prefix, seed));
    fork128.reset();
    fork128.update(&suffix);
    assert_eq!(fork128.digest(), xxh3_128_with_seed(&suffix, seed));

    let secret = secret(SECRET_SIZE_MIN);
    let mut custom = Xxh3::with_secret(&secret).unwrap();
    custom.update(&prefix);
    let custom_fork = custom.clone();
    custom.update(&suffix);
    assert_eq!(
        custom.digest(),
        xxh3_64_with_secret(&joined, &secret).unwrap()
    );
    assert_eq!(
        custom_fork.digest(),
        xxh3_64_with_secret(&prefix, &secret).unwrap()
    );
    custom.reset();
    custom.update(&suffix);
    assert_eq!(custom.seed(), 0);
    assert_eq!(custom.total_len(), suffix.len() as u64);
    assert_eq!(
        custom.digest(),
        xxh3_64_with_secret(&suffix, &secret).unwrap()
    );
}

#[test]
fn standard_hash_traits_use_the_raw_stream() {
    let bytes = input(4_097);

    let mut via_trait32 = Xxh32Builder::with_seed(7).build_hasher();
    via_trait32.write(&bytes);
    assert_eq!(via_trait32.finish(), u64::from(xxh32(&bytes, 7)));

    let mut via_trait64 = Xxh64Builder::with_seed(11).build_hasher();
    via_trait64.write(&bytes);
    assert_eq!(via_trait64.finish(), xxh64(&bytes, 11));

    let mut via_trait3 = Xxh3Builder::with_seed(13).build_hasher();
    via_trait3.write(&bytes);
    assert_eq!(via_trait3.finish(), xxh3_64_with_seed(&bytes, 13));

    let secret = secret(SECRET_SIZE_MIN);
    let mut via_secret = Xxh3SecretBuilder::with_secret(&secret)
        .unwrap()
        .build_hasher();
    via_secret.write(&bytes);
    assert_eq!(
        via_secret.finish(),
        xxh3_64_with_secret(&bytes, &secret).unwrap()
    );

    let mut via_seed_and_secret = Xxh3SecretBuilder::with_seed_and_secret(17, &secret)
        .unwrap()
        .build_hasher();
    via_seed_and_secret.write(&bytes);
    assert_eq!(
        via_seed_and_secret.finish(),
        xxh3_64_with_seed_and_secret(&bytes, 17, &secret).unwrap()
    );

    assert!(rache::kernel::selected_backend().is_available());
}

#[test]
fn family_and_compatibility_module_paths_match() {
    let bytes = input(257);

    assert_eq!(
        rache::xxhash::xxh32(&bytes, 7),
        rache::xxh32::xxh32(&bytes, 7)
    );
    assert_eq!(
        rache::xxhash::xxh64(&bytes, 11),
        rache::xxh64::xxh64(&bytes, 11)
    );
    assert_eq!(rache::xxhash::xxh3_64(&bytes), rache::xxh3::xxh3_64(&bytes));
}
