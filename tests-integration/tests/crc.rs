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

mod support;

use support::next_random;
use support::random_input;

fn ieee_combine(left: u32, right: u32, right_len: u64) -> u32 {
    let mut state = crc32fast::Hasher::new_with_initial(left);
    let suffix = crc32fast::Hasher::new_with_initial_len(right, right_len);
    state.combine(&suffix);
    state.finalize()
}

fn iscsi_combine(left: u32, right: u32, right_len: u64) -> u32 {
    crc32c::crc32c_combine(left, right, right_len.try_into().unwrap())
}

macro_rules! variant_tests {
    ($module:ident, $state:path, $checksum:path, $combine:path, $algorithm:path, $specialized:path, $reference_combine:path) => {
        mod $module {
            use $checksum as checksum;
            use $combine as combine;
            use $reference_combine as reference_combine;
            use $specialized as specialized;
            use $state as State;

            use super::*;

            const REFERENCE: crc::Crc<u32> = crc::Crc::<u32>::new(&$algorithm);

            #[test]
            fn byte_boundaries_and_resumed_prefixes() {
                let mut random = 0x51ed_270b_9e37_79b9;
                let bytes = random_input(&mut random, 272);
                for len in 0..=256 {
                    let offset = len % 16;
                    let input = &bytes[offset..offset + len];
                    let expected = REFERENCE.checksum(input);
                    assert_eq!(checksum(input), expected, "length={len}");
                    assert_eq!(specialized(input), expected, "reference length={len}");
                    for split in 0..=len {
                        let mut state = State::new();
                        state.update(&input[..split]);
                        let prefix = state.digest();
                        assert_eq!(prefix, REFERENCE.checksum(&input[..split]));
                        let saved = state.clone();
                        state.update(&[]);
                        assert_eq!(state.digest(), prefix);
                        state.update(&input[split..]);
                        assert_eq!(state.digest(), expected, "length={len} split={split}");
                        assert_eq!(saved.digest(), prefix);

                        let mut resumed = State::from_digest(prefix);
                        resumed.update(&input[split..]);
                        assert_eq!(resumed.digest(), expected);
                    }
                    let mut state = State::from_digest(expected);
                    state.reset();
                    assert_eq!(state.digest(), 0);
                    state.update(input);
                    assert_eq!(state.digest(), expected);
                }
            }

            #[test]
            fn fragmented_payloads_and_zeroed_checksum_fields() {
                let mut random = 0xa409_3822_299f_31d0;
                for case in 0..64 {
                    let len = next_random(&mut random) as usize % (64 * 1_024);
                    let bytes = random_input(&mut random, len);
                    let mut state = State::new();
                    let mut reference = REFERENCE.digest();
                    let mut offset = 0;
                    while offset < len {
                        let end = (offset + 1 + next_random(&mut random) as usize % 257).min(len);
                        state.update(&bytes[offset..end]);
                        reference.update(&bytes[offset..end]);
                        assert_eq!(
                            state.digest(),
                            reference.clone().finalize(),
                            "case={case} offset={end}"
                        );
                        offset = end;
                    }
                    assert_eq!(state.digest(), specialized(&bytes));
                }

                let mut page = random_input(&mut random, 4 * 1_024);
                for offset in [0, 7, 56, page.len() - 4] {
                    let mut state = State::new();
                    state.update(&page[..offset]);
                    state.update(&[0; 4]);
                    state.update(&page[offset + 4..]);
                    page[offset..offset + 4].fill(0);
                    assert_eq!(state.digest(), REFERENCE.checksum(&page));
                }
            }

            #[test]
            fn combining_segments_preserves_concatenation_and_order() {
                let mut random = 0x082e_fa98_ec4e_6c89;
                let bytes = random_input(&mut random, 4 * 1_024);
                for len in [0, 1, 7, 8, 9, 31, 32, 255, 256, 1_024, 4 * 1_024] {
                    let input = &bytes[..len];
                    let expected = REFERENCE.checksum(input);
                    for split in [0, len / 3, len / 2, len.saturating_sub(1), len] {
                        let left = checksum(&input[..split]);
                        let right = checksum(&input[split..]);
                        let right_len = (len - split) as u64;
                        assert_eq!(
                            combine(left, right, right_len),
                            expected,
                            "len={len} split={split}"
                        );
                        assert_eq!(
                            combine(left, right, right_len),
                            reference_combine(left, right, right_len)
                        );
                    }
                    let a = len / 3;
                    let b = len * 2 / 3;
                    let ab = combine(
                        checksum(&input[..a]),
                        checksum(&input[a..b]),
                        (b - a) as u64,
                    );
                    let bc = combine(
                        checksum(&input[a..b]),
                        checksum(&input[b..]),
                        (len - b) as u64,
                    );
                    assert_eq!(
                        combine(ab, checksum(&input[b..]), (len - b) as u64),
                        expected
                    );
                    assert_eq!(
                        combine(checksum(&input[..a]), bc, (len - a) as u64),
                        expected
                    );
                }
            }

            #[test]
            fn combine_supports_lengths_beyond_four_gibibytes() {
                // Build reference checksums of power-of-two zero runs without allocating the runs.
                let prefix = checksum(b"prefix");
                let mut zeros = specialized(&[0]);
                for bit in 0..64 {
                    let len = 1_u64 << bit;
                    if bit >= 31 && len <= usize::MAX as u64 {
                        assert_eq!(
                            combine(prefix, zeros, len),
                            reference_combine(prefix, zeros, len),
                            "length={len}"
                        );
                        let with_tail = reference_combine(zeros, specialized(&[0; 7]), 7);
                        assert_eq!(
                            combine(prefix, with_tail, len + 7),
                            reference_combine(prefix, with_tail, len + 7),
                            "length={} with a short tail",
                            len + 7
                        );
                    }
                    if bit < 63 && len <= usize::MAX as u64 {
                        zeros = reference_combine(zeros, zeros, len);
                    }
                }
            }
        }
    };
}

variant_tests!(
    iso_hdlc,
    hashcrew::crc::Crc32IsoHdlc,
    hashcrew::crc::crc32_iso_hdlc,
    hashcrew::crc::crc32_iso_hdlc_combine,
    crc::CRC_32_ISO_HDLC,
    crc32fast::hash,
    ieee_combine
);
variant_tests!(
    iscsi,
    hashcrew::crc::Crc32Iscsi,
    hashcrew::crc::crc32_iscsi,
    hashcrew::crc::crc32_iscsi_combine,
    crc::CRC_32_ISCSI,
    crc32c::crc32c,
    iscsi_combine
);
