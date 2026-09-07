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

use hashcrew::md5::Md5;
use hashcrew::md5::md5;
use md5::Digest;
use md5::Md5 as ReferenceMd5;

mod support;

use support::next_random;
use support::random_input;

#[test]
fn padding_and_block_boundaries_match_reference() {
    let mut random = 0x6a09_e667_f3bc_c909;
    let bytes = random_input(&mut random, 264);
    for len in 0..=256 {
        let offset = len % 8;
        let input = &bytes[offset..offset + len];
        let expected: [u8; 16] = ReferenceMd5::digest(input).into();
        assert_eq!(md5(input), expected, "one-shot length={len}");

        for split in 0..=len {
            let mut state = Md5::new();
            state.update(&input[..split]);
            let prefix: [u8; 16] = ReferenceMd5::digest(&input[..split]).into();
            assert_eq!(state.digest(), prefix, "prefix length={len} split={split}");
            state.update(&[]);
            state.update(&input[split..]);
            assert_eq!(
                state.digest(),
                expected,
                "stream length={len} split={split}"
            );
        }
    }
}

#[test]
fn randomized_streams_match_reference() {
    let mut random = 0xbb67_ae85_84ca_a73b;
    let mut state = Md5::new();
    for case in 0..128 {
        let len = next_random(&mut random) as usize % (128 * 1_024);
        let bytes = random_input(&mut random, len);
        let expected: [u8; 16] = ReferenceMd5::digest(&bytes).into();
        assert_eq!(md5(&bytes), expected, "one-shot case={case} length={len}");

        state.reset();
        let mut reference = ReferenceMd5::new();
        let mut offset = 0;
        while offset < len {
            let end = (offset + 1 + next_random(&mut random) as usize % 1_024).min(len);
            state.update(&bytes[offset..end]);
            reference.update(&bytes[offset..end]);
            let prefix: [u8; 16] = reference.clone().finalize().into();
            assert_eq!(state.digest(), prefix, "stream case={case} offset={end}");
            offset = end;
        }
        assert_eq!(state.digest(), expected, "stream case={case} length={len}");
    }
}
