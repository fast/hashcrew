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

use core::hash::Hasher as _;

use divan::Bencher;
use divan::black_box;
use divan::counter::BytesCount;

mod support;

use support::input;

fn main() {
    divan::main();
}

const CASES: &[(usize, usize)] = &[
    (32, 8),
    (241, 17),
    (4 * 1_024, 7),
    (4 * 1_024, 64),
    (4 * 1_024, 1_024),
    (64 * 1_024, 1_024),
    (1_024 * 1_024, 64 * 1_024),
];

mod crc32_iso_hdlc {
    use super::*;

    const REFERENCE: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
    const REFERENCE_16: crc::Crc<u32, crc::Table<16>> =
        crc::Crc::<u32, crc::Table<16>>::new(&crc::CRC_32_ISO_HDLC);

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = hashcrew::crc::Crc32IsoHdlc::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn crc_table_1(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = REFERENCE.digest();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.finalize()
        });
    }

    #[divan::bench(args = CASES)]
    fn crc_table_16(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = REFERENCE_16.digest();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.finalize()
        });
    }

    #[divan::bench(args = CASES)]
    fn crc32fast(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = crc32fast::Hasher::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.finalize()
        });
    }

    #[cfg(feature = "crc-fast")]
    #[divan::bench(args = CASES)]
    fn crc_fast(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = crc_fast::Digest::new(crc_fast::CrcAlgorithm::Crc32IsoHdlc);
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.finalize()
        });
    }
}

mod crc32_iscsi {
    use super::*;

    const REFERENCE: crc::Crc<u32> = crc::Crc::<u32>::new(&crc::CRC_32_ISCSI);
    const REFERENCE_16: crc::Crc<u32, crc::Table<16>> =
        crc::Crc::<u32, crc::Table<16>>::new(&crc::CRC_32_ISCSI);

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = hashcrew::crc::Crc32Iscsi::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn crc_table_1(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = REFERENCE.digest();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.finalize()
        });
    }

    #[divan::bench(args = CASES)]
    fn crc_table_16(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = REFERENCE_16.digest();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.finalize()
        });
    }

    #[divan::bench(args = CASES)]
    fn crc32c(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut checksum = 0;
            for chunk in black_box(&bytes).chunks(chunk_size) {
                checksum = crc32c::crc32c_append(checksum, chunk);
            }
            checksum
        });
    }

    #[cfg(feature = "crc-fast")]
    #[divan::bench(args = CASES)]
    fn crc_fast(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = crc_fast::Digest::new(crc_fast::CrcAlgorithm::Crc32Iscsi);
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.finalize()
        });
    }
}

mod md5 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = hashcrew::md5::Md5::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn rustcrypto(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        use ::md5::Digest;
        use ::md5::Md5;

        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut state = Md5::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                state.update(chunk);
            }
            state.finalize()
        });
    }

    mod digest {
        use super::*;

        const LENGTHS: &[usize] = &[0, 32, 55, 56, 63, 64];

        #[divan::bench(args = LENGTHS)]
        fn hashcrew(bencher: Bencher<'_, '_>, len: usize) {
            let mut state = hashcrew::md5::Md5::new();
            state.update(&input(len));
            // Prepare fresh states outside timing, as for consuming finalization below.
            bencher
                .with_inputs(|| state.clone())
                .bench_values(|state| state.digest());
        }

        #[divan::bench(args = LENGTHS)]
        fn rustcrypto(bencher: Bencher<'_, '_>, len: usize) {
            use ::md5::Digest;
            use ::md5::Md5;

            let mut state = Md5::new();
            state.update(input(len));
            bencher
                .with_inputs(|| state.clone())
                .bench_values(|state| state.finalize());
        }
    }
}

mod xxh32 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::xxhash::Xxh32::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = xxhash_rust::xxh32::Xxh32::new(0);
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn twox_hash(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = twox_hash::xxhash32::Hasher::with_seed(0);
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.write(chunk);
            }
            hasher.finish_32()
        });
    }
}

mod xxh64 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::xxhash::Xxh64::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = xxhash_rust::xxh64::Xxh64::new(0);
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn twox_hash(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = twox_hash::xxhash64::Hasher::with_seed(0);
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.write(chunk);
            }
            hasher.finish()
        });
    }
}

mod xxh3_64 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::xxhash::Xxh3_64::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn hashcrew_secret(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        let secret = input(hashcrew::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::xxhash::Xxh3_64::with_secret(black_box(&secret)).unwrap();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn hashcrew_seed_and_secret(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        let secret = input(hashcrew::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::xxhash::Xxh3_64::with_seed_and_secret(
                0x0123_4567_89ab_cdef,
                black_box(&secret),
            )
            .unwrap();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = xxhash_rust::xxh3::Xxh3Default::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn twox_hash(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = twox_hash::xxhash3_64::Hasher::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.write(chunk);
            }
            hasher.finish()
        });
    }
}

mod xxh3_128 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::xxhash::Xxh3_128::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn hashcrew_secret(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        let secret = input(hashcrew::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::xxhash::Xxh3_128::with_secret(black_box(&secret)).unwrap();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn hashcrew_seed_and_secret(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        let secret = input(hashcrew::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::xxhash::Xxh3_128::with_seed_and_secret(
                0x0123_4567_89ab_cdef,
                black_box(&secret),
            )
            .unwrap();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = xxhash_rust::xxh3::Xxh3Default::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest128()
        });
    }

    #[divan::bench(args = CASES)]
    fn twox_hash(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = twox_hash::xxhash3_128::Hasher::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.write(chunk);
            }
            hasher.finish_128()
        });
    }
}

mod murmur3_x86_32 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::murmur::Murmur3X86_32::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }
}

mod murmur3_x86_128 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::murmur::Murmur3X86_128::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }
}

mod murmur3_x64_128 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::murmur::Murmur3X64_128::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }
}

mod fnv1a_32 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::fnv::Fnv1a32::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }
}

mod fnv1a_64 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn hashcrew(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = hashcrew::fnv::Fnv1a64::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn fnv_crate(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = fnv::FnvHasher::default();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.write(chunk);
            }
            hasher.finish()
        });
    }
}
