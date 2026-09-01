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
use std::io::Cursor;

use divan::Bencher;
use divan::black_box;
use divan::counter::BytesCount;

mod support;

use support::input;

fn main() {
    divan::main();
}

const SIZES: &[usize] = &[
    1,
    3,
    4,
    8,
    9,
    16,
    17,
    32,
    64,
    128,
    240,
    241,
    256,
    1_024,
    4 * 1_024,
    64 * 1_024,
    1_024 * 1_024,
];
const SEED: u64 = 0x0123_4567_89ab_cdef;

mod cityhash32 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::cityhash32(black_box(&bytes)));
    }

    #[divan::bench(args = SIZES)]
    fn cityhasher(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| cityhasher::hash::<u32>(black_box(bytes.as_slice())));
    }
}

mod cityhash64 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::cityhash64(black_box(&bytes)));
    }

    #[divan::bench(args = SIZES)]
    fn cityhasher(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| cityhasher::hash::<u64>(black_box(bytes.as_slice())));
    }
}

mod cityhash64_seeded {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache_one_seed(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::cityhash64_with_seed(black_box(&bytes), SEED));
    }

    #[divan::bench(args = SIZES)]
    fn rache_two_seeds(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::cityhash64_with_seeds(black_box(&bytes), SEED, !SEED));
    }

    #[divan::bench(args = SIZES)]
    fn cityhasher(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| cityhasher::hash_with_seed::<u64>(black_box(bytes.as_slice()), SEED));
    }
}

mod cityhash128 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::cityhash128(black_box(&bytes)));
    }

    #[divan::bench(args = SIZES)]
    fn cityhash_rs(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| cityhash_rs::cityhash_110_128(black_box(&bytes)));
    }
}

mod cityhash128_seeded {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let seed = (u128::from(!SEED) << 64) | u128::from(SEED);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::cityhash128_with_seed(black_box(&bytes), seed));
    }
}

mod xxh32 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::xxh32(black_box(&bytes), 0));
    }

    #[divan::bench(args = SIZES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| xxhash_rust::xxh32::xxh32(black_box(&bytes), 0));
    }

    #[divan::bench(args = SIZES)]
    fn twox_hash(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| twox_hash::xxhash32::Hasher::oneshot(0, black_box(&bytes)));
    }
}

mod xxh64 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::xxh64(black_box(&bytes), 0));
    }

    #[divan::bench(args = SIZES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| xxhash_rust::xxh64::xxh64(black_box(&bytes), 0));
    }

    #[divan::bench(args = SIZES)]
    fn twox_hash(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| twox_hash::xxhash64::Hasher::oneshot(0, black_box(&bytes)));
    }
}

mod xxh3_64 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::xxh3_64(black_box(&bytes)));
    }

    #[divan::bench(args = SIZES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| xxhash_rust::xxh3::xxh3_64(black_box(&bytes)));
    }

    #[divan::bench(args = SIZES)]
    fn twox_hash(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| twox_hash::xxhash3_64::Hasher::oneshot(black_box(&bytes)));
    }
}

mod xxh3_128 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::xxh3_128(black_box(&bytes)));
    }

    #[divan::bench(args = SIZES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| xxhash_rust::xxh3::xxh3_128(black_box(&bytes)));
    }

    #[divan::bench(args = SIZES)]
    fn twox_hash(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| twox_hash::xxhash3_128::Hasher::oneshot(black_box(&bytes)));
    }
}

mod xxh3_64_seeded {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::xxh3_64_with_seed(black_box(&bytes), SEED));
    }

    #[divan::bench(args = SIZES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| xxhash_rust::xxh3::xxh3_64_with_seed(black_box(&bytes), SEED));
    }

    #[divan::bench(args = SIZES)]
    fn twox_hash(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| twox_hash::xxhash3_64::Hasher::oneshot_with_seed(SEED, black_box(&bytes)));
    }
}

mod xxh3_128_seeded {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::xxh3_128_with_seed(black_box(&bytes), SEED));
    }

    #[divan::bench(args = SIZES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| xxhash_rust::xxh3::xxh3_128_with_seed(black_box(&bytes), SEED));
    }

    #[divan::bench(args = SIZES)]
    fn twox_hash(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| twox_hash::xxhash3_128::Hasher::oneshot_with_seed(SEED, black_box(&bytes)));
    }
}

mod xxh3_64_secret {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let secret = input(rache::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            rache::raw::xxh3_64_with_secret(black_box(&bytes), black_box(&secret)).unwrap()
        });
    }

    #[divan::bench(args = SIZES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let secret = input(rache::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            xxhash_rust::xxh3::xxh3_64_with_secret(black_box(&bytes), black_box(&secret))
        });
    }

    #[divan::bench(args = SIZES)]
    fn twox_hash(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let secret = input(rache::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            twox_hash::xxhash3_64::Hasher::oneshot_with_secret(
                black_box(&secret),
                black_box(&bytes),
            )
            .unwrap()
        });
    }
}

mod xxh3_128_secret {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let secret = input(rache::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            rache::raw::xxh3_128_with_secret(black_box(&bytes), black_box(&secret)).unwrap()
        });
    }

    #[divan::bench(args = SIZES)]
    fn xxhash_rust(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let secret = input(rache::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            xxhash_rust::xxh3::xxh3_128_with_secret(black_box(&bytes), black_box(&secret))
        });
    }

    #[divan::bench(args = SIZES)]
    fn twox_hash(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let secret = input(rache::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            twox_hash::xxhash3_128::Hasher::oneshot_with_secret(
                black_box(&secret),
                black_box(&bytes),
            )
            .unwrap()
        });
    }
}

mod xxh3_64_seed_and_secret {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let secret = input(rache::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            rache::raw::xxh3_64_with_seed_and_secret(black_box(&bytes), SEED, black_box(&secret))
                .unwrap()
        });
    }

    #[divan::bench(args = SIZES)]
    fn twox_hash(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let secret = input(rache::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            twox_hash::xxhash3_64::Hasher::oneshot_with_seed_and_secret(
                SEED,
                black_box(&secret),
                black_box(&bytes),
            )
            .unwrap()
        });
    }
}

mod xxh3_128_seed_and_secret {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let secret = input(rache::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            rache::raw::xxh3_128_with_seed_and_secret(black_box(&bytes), SEED, black_box(&secret))
                .unwrap()
        });
    }

    #[divan::bench(args = SIZES)]
    fn twox_hash(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        let secret = input(rache::xxhash::DEFAULT_SECRET_SIZE);
        bencher.counter(BytesCount::new(len)).bench(|| {
            twox_hash::xxhash3_128::Hasher::oneshot_with_seed_and_secret(
                SEED,
                black_box(&secret),
                black_box(&bytes),
            )
            .unwrap()
        });
    }
}

mod murmur3_32 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::murmur3_32(black_box(&bytes), 0));
    }

    #[divan::bench(args = SIZES)]
    fn murmur3_crate(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            murmur3::murmur3_32(&mut Cursor::new(black_box(bytes.as_slice())), 0).unwrap()
        });
    }
}

mod murmur3_128 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::murmur3_128(black_box(&bytes), 0));
    }

    #[divan::bench(args = SIZES)]
    fn murmur3_crate(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            murmur3::murmur3_x64_128(&mut Cursor::new(black_box(bytes.as_slice())), 0).unwrap()
        });
    }
}

mod fnv1a_32 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::fnv1a_32(black_box(&bytes)));
    }
}

mod fnv1a_64 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::raw::fnv1a_64(black_box(&bytes)));
    }

    #[divan::bench(args = SIZES)]
    fn fnv_crate(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher.counter(BytesCount::new(len)).bench(|| {
            let mut hash = fnv::FnvHasher::default();
            hash.write(black_box(&bytes));
            hash.finish()
        });
    }
}
