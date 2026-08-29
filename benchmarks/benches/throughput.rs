use divan::counter::BytesCount;
use divan::{Bencher, black_box};

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

fn input(len: usize) -> Vec<u8> {
    (0..len)
        .map(|index| index.wrapping_mul(131).wrapping_add(17) as u8)
        .collect()
}

mod xxh32 {
    use super::*;

    #[divan::bench(args = SIZES)]
    fn rache(bencher: Bencher<'_, '_>, len: usize) {
        let bytes = input(len);
        bencher
            .counter(BytesCount::new(len))
            .bench(|| rache::xxh32(black_box(&bytes), 0));
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
            .bench(|| rache::xxh64(black_box(&bytes), 0));
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
            .bench(|| rache::xxh3_64(black_box(&bytes)));
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
            .bench(|| rache::xxh3_128(black_box(&bytes)));
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
            .bench(|| rache::xxh3_64_with_seed(black_box(&bytes), SEED));
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
            .bench(|| rache::xxh3_128_with_seed(black_box(&bytes), SEED));
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
