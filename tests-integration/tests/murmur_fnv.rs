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

use core::hash::BuildHasher;
use core::hash::Hasher;
use std::io::Cursor;

use rache::fnv::Fnv1a32;
use rache::fnv::Fnv1a32Builder;
use rache::fnv::Fnv1a64;
use rache::fnv::Fnv1a64Builder;
use rache::fnv::fnv1a_32;
use rache::fnv::fnv1a_32_with_offset_basis;
use rache::fnv::fnv1a_64;
use rache::fnv::fnv1a_64_with_offset_basis;
use rache::murmur::Murmur3_32;
use rache::murmur::Murmur3_32Builder;
use rache::murmur::Murmur3X64_128;
use rache::murmur::Murmur3X86_128;
use rache::murmur::murmur3_32;
use rache::murmur::murmur3_x64_128;
use rache::murmur::murmur3_x86_128;

mod support;

use support::next_random;
use support::random_input;

const LENGTHS: &[usize] = &[
    0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 31, 32, 33, 63, 64, 65, 127, 128,
    129, 255, 256, 257, 1_023, 1_024, 1_025, 4_097,
];
const SEEDS: &[u32] = &[0, 1, 0x89ab_cdef, u32::MAX];

fn input(len: usize) -> Vec<u8> {
    (0..len)
        .map(|index| index.wrapping_mul(131).wrapping_add(17) as u8)
        .collect()
}

fn reference_fnv1a_64(input: &[u8]) -> u64 {
    let mut hash = fnv::FnvHasher::default();
    hash.write(input);
    hash.finish()
}

fn reference_fnv1a_64_with_offset_basis(input: &[u8], offset_basis: u64) -> u64 {
    let mut hash = fnv::FnvHasher::with_key(offset_basis);
    hash.write(input);
    hash.finish()
}

#[test]
fn murmur_oneshot_matches_reference() {
    for &len in LENGTHS {
        let bytes = input(len);
        for &seed in SEEDS {
            assert_eq!(
                murmur3_32(&bytes, seed),
                murmur3::murmur3_32(&mut Cursor::new(&bytes), seed).unwrap(),
                "MurmurHash3 x86_32 length={len} seed={seed:#x}"
            );
            assert_eq!(
                murmur3_x86_128(&bytes, seed),
                murmur3::murmur3_x86_128(&mut Cursor::new(&bytes), seed).unwrap(),
                "MurmurHash3 x86_128 length={len} seed={seed:#x}"
            );
            assert_eq!(
                murmur3_x64_128(&bytes, seed),
                murmur3::murmur3_x64_128(&mut Cursor::new(&bytes), seed).unwrap(),
                "MurmurHash3 x64_128 length={len} seed={seed:#x}"
            );
        }
    }
}

#[test]
fn every_length_through_two_kib_matches_murmur_reference() {
    let bytes = input(2 * 1_024);
    for len in 0..=bytes.len() {
        let input = &bytes[..len];
        let seed = (len as u32)
            .wrapping_mul(0x9e37_79b1)
            .rotate_left((len % 32) as u32);

        assert_eq!(
            murmur3_32(input, seed),
            murmur3::murmur3_32(&mut Cursor::new(input), seed).unwrap(),
            "MurmurHash3 x86_32 length={len} seed={seed:#x}"
        );
        assert_eq!(
            murmur3_x86_128(input, seed),
            murmur3::murmur3_x86_128(&mut Cursor::new(input), seed).unwrap(),
            "MurmurHash3 x86_128 length={len} seed={seed:#x}"
        );
        assert_eq!(
            murmur3_x64_128(input, seed),
            murmur3::murmur3_x64_128(&mut Cursor::new(input), seed).unwrap(),
            "MurmurHash3 x64_128 length={len} seed={seed:#x}"
        );
    }
}

#[test]
fn randomized_murmur_inputs_match_reference() {
    let mut random = 0x6a09_e667_f3bc_c909;
    for case in 0..256 {
        let len = next_random(&mut random) as usize % (128 * 1_024);
        let seed = next_random(&mut random) as u32;
        let bytes = random_input(&mut random, len);

        assert_eq!(
            murmur3_32(&bytes, seed),
            murmur3::murmur3_32(&mut Cursor::new(&bytes), seed).unwrap(),
            "MurmurHash3 x86_32 case={case} length={len}"
        );
        assert_eq!(
            murmur3_x86_128(&bytes, seed),
            murmur3::murmur3_x86_128(&mut Cursor::new(&bytes), seed).unwrap(),
            "MurmurHash3 x86_128 case={case} length={len}"
        );
        assert_eq!(
            murmur3_x64_128(&bytes, seed),
            murmur3::murmur3_x64_128(&mut Cursor::new(&bytes), seed).unwrap(),
            "MurmurHash3 x64_128 case={case} length={len}"
        );
    }
}

#[test]
fn fnv_matches_specification_and_reference() {
    let vectors = [
        (b"".as_slice(), 0x811c_9dc5, 0xcbf2_9ce4_8422_2325),
        (b"a".as_slice(), 0xe40c_292c, 0xaf63_dc4c_8601_ec8c),
        (b"foobar".as_slice(), 0xbf9c_f968, 0x8594_4171_f739_67e8),
        (
            b"Hello!\x01\xff\xed".as_slice(),
            0xfd9d_3881,
            0xbd51_ea70_94ee_6fa1,
        ),
    ];
    for (bytes, expected32, expected64) in vectors {
        assert_eq!(fnv1a_32(bytes), expected32);
        assert_eq!(fnv1a_64(bytes), expected64);
        assert_eq!(fnv1a_64(bytes), reference_fnv1a_64(bytes));
    }

    for &len in LENGTHS {
        let bytes = input(len);
        assert_eq!(fnv1a_64(&bytes), reference_fnv1a_64(&bytes));
    }
}

#[test]
fn fnv_custom_offset_basis_matches_chaining_and_reference() {
    let prefix = b"rache namespace";
    let suffix = input(4_097);
    let mut joined = prefix.to_vec();
    joined.extend_from_slice(&suffix);
    let basis32 = fnv1a_32(prefix);
    let basis64 = fnv1a_64(prefix);

    assert_eq!(
        fnv1a_32_with_offset_basis(&suffix, basis32),
        fnv1a_32(&joined)
    );
    assert_eq!(
        fnv1a_64_with_offset_basis(&suffix, basis64),
        fnv1a_64(&joined)
    );

    for basis in [0, 1, basis64, 0x0123_4567_89ab_cdef, u64::MAX] {
        assert_eq!(
            fnv1a_64_with_offset_basis(&suffix, basis),
            reference_fnv1a_64_with_offset_basis(&suffix, basis),
            "offset_basis={basis:#x}"
        );
    }

    let mut hash32 = Fnv1a32::with_offset_basis(basis32);
    let mut hash64 = Fnv1a64::with_offset_basis(basis64);
    for chunk in suffix.chunks(31) {
        hash32.update(chunk);
        hash64.update(chunk);
    }
    assert_eq!(hash32.offset_basis(), basis32);
    assert_eq!(hash64.offset_basis(), basis64);
    assert_eq!(
        hash32.digest(),
        fnv1a_32_with_offset_basis(&suffix, basis32)
    );
    assert_eq!(
        hash64.digest(),
        fnv1a_64_with_offset_basis(&suffix, basis64)
    );

    hash32.reset();
    hash64.reset();
    hash32.update(&suffix);
    hash64.update(&suffix);
    assert_eq!(
        hash32.digest(),
        fnv1a_32_with_offset_basis(&suffix, basis32)
    );
    assert_eq!(
        hash64.digest(),
        fnv1a_64_with_offset_basis(&suffix, basis64)
    );
}

#[test]
fn randomized_streaming_partitions_match_oneshot() {
    let mut random = 0xbb67_ae85_84ca_a73b;
    for case in 0..128 {
        let len = next_random(&mut random) as usize % (64 * 1_024);
        let seed = next_random(&mut random) as u32;
        let bytes = random_input(&mut random, len);
        let mut murmur32 = Murmur3_32::with_seed(seed);
        let mut murmur_x86_128 = Murmur3X86_128::with_seed(seed);
        let mut murmur_x64_128 = Murmur3X64_128::with_seed(seed);
        let mut fnv32 = Fnv1a32::new();
        let mut fnv64 = Fnv1a64::new();
        let mut offset = 0;

        while offset < bytes.len() {
            let chunk_len = 1 + next_random(&mut random) as usize % 521;
            let end = (offset + chunk_len).min(bytes.len());
            let chunk = &bytes[offset..end];
            murmur32.update(chunk);
            murmur_x86_128.update(chunk);
            murmur_x64_128.update(chunk);
            fnv32.update(chunk);
            fnv64.update(chunk);
            if next_random(&mut random) & 3 == 0 {
                murmur32.update(&[]);
                murmur_x86_128.update(&[]);
                murmur_x64_128.update(&[]);
                fnv32.update(&[]);
                fnv64.update(&[]);
            }
            offset = end;
        }

        assert_eq!(
            murmur32.digest(),
            murmur3_32(&bytes, seed),
            "MurmurHash3 x86_32 case={case}"
        );
        assert_eq!(
            murmur_x86_128.digest(),
            murmur3_x86_128(&bytes, seed),
            "MurmurHash3 x86_128 case={case}"
        );
        assert_eq!(
            murmur_x64_128.digest(),
            murmur3_x64_128(&bytes, seed),
            "MurmurHash3 x64_128 case={case}"
        );
        assert_eq!(fnv32.digest(), fnv1a_32(&bytes), "FNV-1a 32 case={case}");
        assert_eq!(fnv64.digest(), fnv1a_64(&bytes), "FNV-1a 64 case={case}");
    }
}

#[test]
fn every_two_way_partition_matches_oneshot() {
    let bytes = input(257);
    for len in 0..=bytes.len() {
        let input = &bytes[..len];
        let seed = (len as u32)
            .wrapping_mul(0x9e37_79b1)
            .rotate_left((len % 32) as u32);
        let expected_murmur32 = murmur3_32(input, seed);
        let expected_murmur_x86_128 = murmur3_x86_128(input, seed);
        let expected_murmur_x64_128 = murmur3_x64_128(input, seed);
        let expected_fnv32 = fnv1a_32(input);
        let expected_fnv64 = fnv1a_64(input);

        for split in 0..=len {
            let mut murmur32 = Murmur3_32::with_seed(seed);
            let mut murmur_x86_128 = Murmur3X86_128::with_seed(seed);
            let mut murmur_x64_128 = Murmur3X64_128::with_seed(seed);
            let mut fnv32 = Fnv1a32::new();
            let mut fnv64 = Fnv1a64::new();

            for chunk in [&[][..], &input[..split], &[][..], &input[split..], &[][..]] {
                murmur32.update(chunk);
                murmur_x86_128.update(chunk);
                murmur_x64_128.update(chunk);
                fnv32.update(chunk);
                fnv64.update(chunk);
            }

            assert_eq!(
                murmur32.digest(),
                expected_murmur32,
                "MurmurHash3 x86_32 length={len} split={split}"
            );
            assert_eq!(
                murmur_x86_128.digest(),
                expected_murmur_x86_128,
                "MurmurHash3 x86_128 length={len} split={split}"
            );
            assert_eq!(
                murmur_x64_128.digest(),
                expected_murmur_x64_128,
                "MurmurHash3 x64_128 length={len} split={split}"
            );
            assert_eq!(
                fnv32.digest(),
                expected_fnv32,
                "FNV-1a 32 length={len} split={split}"
            );
            assert_eq!(
                fnv64.digest(),
                expected_fnv64,
                "FNV-1a 64 length={len} split={split}"
            );
        }
    }
}

#[test]
fn streaming_states_can_be_cloned_finished_continued_and_reset() {
    let seed = 0x89ab_cdef;
    let prefix = input(1_337);
    let suffix = input(777);
    let joined: Vec<_> = prefix.iter().chain(&suffix).copied().collect();

    let mut murmur = Murmur3X86_128::with_seed(seed);
    murmur.update(&prefix);
    let mut fork = murmur.clone();
    murmur.update(&suffix);
    assert_eq!(murmur.digest(), murmur3_x86_128(&joined, seed));
    assert_eq!(fork.digest(), murmur3_x86_128(&prefix, seed));
    fork.reset();
    fork.update(&suffix);
    assert_eq!(fork.seed(), seed);
    assert_eq!(fork.total_len(), suffix.len() as u64);
    assert_eq!(fork.digest(), murmur3_x86_128(&suffix, seed));

    let mut murmur = Murmur3X64_128::with_seed(seed);
    murmur.update(&prefix);
    let mut fork = murmur.clone();
    murmur.update(&suffix);
    assert_eq!(murmur.digest(), murmur3_x64_128(&joined, seed));
    assert_eq!(fork.digest(), murmur3_x64_128(&prefix, seed));
    fork.reset();
    fork.update(&suffix);
    assert_eq!(fork.seed(), seed);
    assert_eq!(fork.total_len(), suffix.len() as u64);
    assert_eq!(fork.digest(), murmur3_x64_128(&suffix, seed));

    let mut fnv = Fnv1a64::new();
    fnv.update(&prefix);
    let mut fnv_fork = fnv;
    fnv.update(&suffix);
    assert_eq!(fnv.digest(), fnv1a_64(&joined));
    assert_eq!(fnv_fork.digest(), fnv1a_64(&prefix));
    fnv_fork.reset();
    fnv_fork.update(&suffix);
    assert_eq!(fnv_fork.digest(), fnv1a_64(&suffix));
}

#[test]
fn standard_hash_traits_use_the_raw_stream() {
    let bytes = input(4_097);

    let mut murmur = Murmur3_32Builder::with_seed(7).build_hasher();
    murmur.write(&bytes);
    assert_eq!(murmur.finish(), u64::from(murmur3_32(&bytes, 7)));

    let mut fnv32 = Fnv1a32Builder::default().build_hasher();
    fnv32.write(&bytes);
    assert_eq!(fnv32.finish(), u64::from(fnv1a_32(&bytes)));

    let mut fnv64 = Fnv1a64Builder::default().build_hasher();
    fnv64.write(&bytes);
    assert_eq!(fnv64.finish(), fnv1a_64(&bytes));

    let basis32 = 0x0123_4567;
    let mut custom32 = Fnv1a32Builder::with_offset_basis(basis32).build_hasher();
    custom32.write(&bytes);
    assert_eq!(
        custom32.finish(),
        u64::from(fnv1a_32_with_offset_basis(&bytes, basis32))
    );

    let basis64 = 0x0123_4567_89ab_cdef;
    let mut custom64 = Fnv1a64Builder::with_offset_basis(basis64).build_hasher();
    custom64.write(&bytes);
    assert_eq!(
        custom64.finish(),
        fnv1a_64_with_offset_basis(&bytes, basis64)
    );
}
