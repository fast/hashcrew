//! FNV-1a one-shot and streaming APIs.

use core::hash::{BuildHasher, Hasher};

/// Standard 32-bit FNV offset basis.
pub const FNV1A_32_OFFSET_BASIS: u32 = 0x811c_9dc5;
/// Standard 32-bit FNV prime.
pub const FNV1A_32_PRIME: u32 = 0x0100_0193;
/// Standard 64-bit FNV offset basis.
pub const FNV1A_64_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// Standard 64-bit FNV prime.
pub const FNV1A_64_PRIME: u64 = 0x0000_0100_0000_01b3;

#[inline(always)]
fn update_32(mut hash: u32, input: &[u8]) -> u32 {
    for &byte in input {
        hash = (hash ^ u32::from(byte)).wrapping_mul(FNV1A_32_PRIME);
    }
    hash
}

#[inline(always)]
fn update_64(mut hash: u64, input: &[u8]) -> u64 {
    for &byte in input {
        hash = (hash ^ u64::from(byte)).wrapping_mul(FNV1A_64_PRIME);
    }
    hash
}

/// Hashes `input` with 32-bit FNV-1a.
#[must_use]
#[inline]
pub fn fnv1a_32(input: &[u8]) -> u32 {
    update_32(FNV1A_32_OFFSET_BASIS, input)
}

/// Hashes `input` with 64-bit FNV-1a.
#[must_use]
#[inline]
pub fn fnv1a_64(input: &[u8]) -> u64 {
    update_64(FNV1A_64_OFFSET_BASIS, input)
}

/// Incremental 32-bit FNV-1a state.
#[derive(Clone, Copy, Debug)]
pub struct Fnv1a32 {
    hash: u32,
}

impl Fnv1a32 {
    /// Creates a state using the standard offset basis.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hash: FNV1A_32_OFFSET_BASIS,
        }
    }

    /// Hashes a complete byte slice without constructing streaming state.
    #[must_use]
    #[inline]
    pub fn oneshot(input: &[u8]) -> u32 {
        fnv1a_32(input)
    }

    /// Adds raw bytes to the hash state.
    #[inline]
    pub fn update(&mut self, input: &[u8]) {
        self.hash = update_32(self.hash, input);
    }

    /// Returns the digest without consuming the state.
    #[must_use]
    pub const fn digest(&self) -> u32 {
        self.hash
    }

    /// Resets the state to the standard offset basis.
    pub fn reset(&mut self) {
        self.hash = FNV1A_32_OFFSET_BASIS;
    }
}

impl Default for Fnv1a32 {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Fnv1a32 {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }

    #[inline]
    fn finish(&self) -> u64 {
        u64::from(self.digest())
    }
}

/// Deterministic [`BuildHasher`] for [`Fnv1a32`].
///
/// FNV-1a is intended for trusted inputs and is not resistant to deliberate
/// hash-flooding attacks.
#[derive(Clone, Copy, Debug, Default)]
pub struct Fnv1a32Builder;

impl BuildHasher for Fnv1a32Builder {
    type Hasher = Fnv1a32;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Fnv1a32::new()
    }
}

/// Incremental 64-bit FNV-1a state.
#[derive(Clone, Copy, Debug)]
pub struct Fnv1a64 {
    hash: u64,
}

impl Fnv1a64 {
    /// Creates a state using the standard offset basis.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            hash: FNV1A_64_OFFSET_BASIS,
        }
    }

    /// Hashes a complete byte slice without constructing streaming state.
    #[must_use]
    #[inline]
    pub fn oneshot(input: &[u8]) -> u64 {
        fnv1a_64(input)
    }

    /// Adds raw bytes to the hash state.
    #[inline]
    pub fn update(&mut self, input: &[u8]) {
        self.hash = update_64(self.hash, input);
    }

    /// Returns the digest without consuming the state.
    #[must_use]
    pub const fn digest(&self) -> u64 {
        self.hash
    }

    /// Resets the state to the standard offset basis.
    pub fn reset(&mut self) {
        self.hash = FNV1A_64_OFFSET_BASIS;
    }
}

impl Default for Fnv1a64 {
    fn default() -> Self {
        Self::new()
    }
}

impl Hasher for Fnv1a64 {
    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        self.update(bytes);
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.digest()
    }
}

/// Deterministic [`BuildHasher`] for [`Fnv1a64`].
///
/// FNV-1a is intended for trusted inputs and is not resistant to deliberate
/// hash-flooding attacks.
#[derive(Clone, Copy, Debug, Default)]
pub struct Fnv1a64Builder;

impl BuildHasher for Fnv1a64Builder {
    type Hasher = Fnv1a64;

    #[inline]
    fn build_hasher(&self) -> Self::Hasher {
        Fnv1a64::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specification_vectors_match() {
        let vectors = [
            (b"".as_slice(), 0x811c_9dc5, 0xcbf2_9ce4_8422_2325),
            (b"a".as_slice(), 0xe40c_292c, 0xaf63_dc4c_8601_ec8c),
            (b"foobar".as_slice(), 0xbf9c_f968, 0x8594_4171_f739_67e8),
        ];
        for (input, expected32, expected64) in vectors {
            assert_eq!(fnv1a_32(input), expected32);
            assert_eq!(fnv1a_64(input), expected64);
        }
    }

    #[test]
    fn reset_restores_offset_basis() {
        let mut hash = Fnv1a64::new();
        hash.update(b"before");
        hash.reset();
        hash.update(b"after");
        assert_eq!(hash.digest(), fnv1a_64(b"after"));
    }
}
