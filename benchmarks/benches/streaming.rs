use core::hash::Hasher as _;

use divan::counter::BytesCount;
use divan::{Bencher, black_box};

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
