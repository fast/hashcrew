use core::arch::x86_64::*;

use super::Xxh3Kernel;

const PRIME32_1: i64 = 0x9e37_79b1;

#[derive(Clone, Copy)]
pub(crate) struct Sse2(());

impl Sse2 {
    pub(crate) unsafe fn new_unchecked() -> Self {
        Self(())
    }
}

impl Xxh3Kernel for Sse2 {
    fn accumulate(self, acc: &mut [u64; 8], stripe: &[u8; 64], secret: &[u8; 64]) {
        // SAFETY: This type is only constructed after SSE2 feature detection.
        unsafe { accumulate_sse2(acc, stripe, secret) }
    }

    fn scramble(self, acc: &mut [u64; 8], secret: &[u8; 64]) {
        // SAFETY: This type is only constructed after SSE2 feature detection.
        unsafe { scramble_sse2(acc, secret) }
    }
}

#[target_feature(enable = "sse2")]
unsafe fn accumulate_sse2(acc: &mut [u64; 8], stripe: &[u8; 64], secret: &[u8; 64]) {
    let acc_ptr = acc.as_mut_ptr().cast::<__m128i>();
    let stripe_ptr = stripe.as_ptr().cast::<__m128i>();
    let secret_ptr = secret.as_ptr().cast::<__m128i>();

    for index in 0..4 {
        // SAFETY: All pointers address two lanes within fixed-size arrays. Unaligned
        // loads and stores are used, and the caller guarantees SSE2 support.
        unsafe {
            let mut acc_vec = _mm_loadu_si128(acc_ptr.add(index));
            let data = _mm_loadu_si128(stripe_ptr.add(index));
            let key = _mm_xor_si128(data, _mm_loadu_si128(secret_ptr.add(index)));
            let swapped = _mm_shuffle_epi32::<0b01_00_11_10>(data);
            let product = _mm_mul_epu32(key, _mm_srli_epi64::<32>(key));
            acc_vec = _mm_add_epi64(acc_vec, swapped);
            acc_vec = _mm_add_epi64(acc_vec, product);
            _mm_storeu_si128(acc_ptr.add(index), acc_vec);
        }
    }
}

#[target_feature(enable = "sse2")]
unsafe fn scramble_sse2(acc: &mut [u64; 8], secret: &[u8; 64]) {
    let acc_ptr = acc.as_mut_ptr().cast::<__m128i>();
    let secret_ptr = secret.as_ptr().cast::<__m128i>();
    let factor = _mm_set1_epi64x(PRIME32_1);

    for index in 0..4 {
        // SAFETY: All pointers address two lanes within fixed-size arrays. Unaligned
        // loads and stores are used, and the caller guarantees SSE2 support.
        unsafe {
            let mut value = _mm_loadu_si128(acc_ptr.add(index));
            value = _mm_xor_si128(value, _mm_srli_epi64::<47>(value));
            value = _mm_xor_si128(value, _mm_loadu_si128(secret_ptr.add(index)));
            let low_product = _mm_mul_epu32(value, factor);
            let high_product = _mm_mul_epu32(_mm_srli_epi64::<32>(value), factor);
            value = _mm_add_epi64(low_product, _mm_slli_epi64::<32>(high_product));
            _mm_storeu_si128(acc_ptr.add(index), value);
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Avx2(());

impl Avx2 {
    pub(crate) unsafe fn new_unchecked() -> Self {
        Self(())
    }
}

impl Xxh3Kernel for Avx2 {
    fn accumulate(self, acc: &mut [u64; 8], stripe: &[u8; 64], secret: &[u8; 64]) {
        // SAFETY: This type is only constructed after AVX2 feature detection.
        unsafe { accumulate_avx2(acc, stripe, secret) }
    }

    fn scramble(self, acc: &mut [u64; 8], secret: &[u8; 64]) {
        // SAFETY: This type is only constructed after AVX2 feature detection.
        unsafe { scramble_avx2(acc, secret) }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn accumulate_avx2(acc: &mut [u64; 8], stripe: &[u8; 64], secret: &[u8; 64]) {
    let acc_ptr = acc.as_mut_ptr().cast::<__m256i>();
    let stripe_ptr = stripe.as_ptr().cast::<__m256i>();
    let secret_ptr = secret.as_ptr().cast::<__m256i>();

    for index in 0..2 {
        // SAFETY: All pointers address four lanes within fixed-size arrays. Unaligned
        // loads and stores are used, and the caller guarantees AVX2 support.
        unsafe {
            let mut acc_vec = _mm256_loadu_si256(acc_ptr.add(index));
            let data = _mm256_loadu_si256(stripe_ptr.add(index));
            let key = _mm256_xor_si256(data, _mm256_loadu_si256(secret_ptr.add(index)));
            let swapped = _mm256_shuffle_epi32::<0b01_00_11_10>(data);
            let product = _mm256_mul_epu32(key, _mm256_srli_epi64::<32>(key));
            acc_vec = _mm256_add_epi64(acc_vec, swapped);
            acc_vec = _mm256_add_epi64(acc_vec, product);
            _mm256_storeu_si256(acc_ptr.add(index), acc_vec);
        }
    }
}

#[target_feature(enable = "avx2")]
unsafe fn scramble_avx2(acc: &mut [u64; 8], secret: &[u8; 64]) {
    let acc_ptr = acc.as_mut_ptr().cast::<__m256i>();
    let secret_ptr = secret.as_ptr().cast::<__m256i>();
    let factor = _mm256_set1_epi64x(PRIME32_1);

    for index in 0..2 {
        // SAFETY: All pointers address four lanes within fixed-size arrays. Unaligned
        // loads and stores are used, and the caller guarantees AVX2 support.
        unsafe {
            let mut value = _mm256_loadu_si256(acc_ptr.add(index));
            value = _mm256_xor_si256(value, _mm256_srli_epi64::<47>(value));
            value = _mm256_xor_si256(value, _mm256_loadu_si256(secret_ptr.add(index)));
            let low_product = _mm256_mul_epu32(value, factor);
            let high_product = _mm256_mul_epu32(_mm256_srli_epi64::<32>(value), factor);
            value = _mm256_add_epi64(low_product, _mm256_slli_epi64::<32>(high_product));
            _mm256_storeu_si256(acc_ptr.add(index), value);
        }
    }
}
