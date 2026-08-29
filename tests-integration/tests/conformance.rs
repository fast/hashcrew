use core::hash::{BuildHasher, Hasher};

use rache::{
    Xxh3, Xxh3_128, Xxh3Builder, Xxh32, Xxh32Builder, Xxh64, Xxh64Builder, xxh3_64_with_seed,
    xxh3_128_with_seed, xxh32, xxh64,
};
use xxhash_rust::{xxh3, xxh32 as reference32, xxh64 as reference64};

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

fn next_random(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_f491_4f6c_dd1d)
}

fn random_input(state: &mut u64, len: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(len);
    while bytes.len() < len {
        bytes.extend_from_slice(&next_random(state).to_le_bytes());
    }
    bytes.truncate(len);
    bytes
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

    assert!(rache::kernel::selected_backend().is_available());
}
