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
    (4 * 1_024, 64),
    (64 * 1_024, 1_024),
    (1_024 * 1_024, 64 * 1_024),
];

mod xxh32 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn rache(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = rache::Xxh32::new();
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
    fn rache(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = rache::Xxh64::new();
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
    fn rache(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = rache::Xxh3::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn rache_secret(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        let secret = input(rache::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = rache::Xxh3::with_secret(black_box(&secret)).unwrap();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn rache_seed_and_secret(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        let secret = input(rache::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher =
                rache::Xxh3::with_seed_and_secret(0x0123_4567_89ab_cdef, black_box(&secret))
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
    fn rache(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = rache::Xxh3_128::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn rache_secret(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        let secret = input(rache::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = rache::Xxh3_128::with_secret(black_box(&secret)).unwrap();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }

    #[divan::bench(args = CASES)]
    fn rache_seed_and_secret(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        let secret = input(rache::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher =
                rache::Xxh3_128::with_seed_and_secret(0x0123_4567_89ab_cdef, black_box(&secret))
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

mod murmur3_32 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn rache(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = rache::Murmur3_32::new();
            for chunk in black_box(&bytes).chunks(chunk_size) {
                hasher.update(chunk);
            }
            hasher.digest()
        });
    }
}

mod murmur3_128 {
    use super::*;

    #[divan::bench(args = CASES)]
    fn rache(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = rache::Murmur3_128::new();
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
    fn rache(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = rache::Fnv1a32::new();
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
    fn rache(bencher: Bencher<'_, '_>, (len, chunk_size): (usize, usize)) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hasher = rache::Fnv1a64::new();
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
