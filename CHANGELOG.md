# Changelog

## 0.1.0

- Provide dependency-free, allocation-free, and `no_std`-compatible implementations of CityHash32, CityHash64, CityHash128, XXH32, XXH64, XXH3-64, XXH3-128, all three reference MurmurHash3 variants, and FNV-1a 32 and 64.
- Group each algorithm family under `hashcrew::{cityhash, xxhash, murmur, fnv}` with one-shot functions for complete byte slices, bounded-memory streaming states where the algorithm permits them, and matching `Hasher` and `BuildHasher` adapters for 32-bit and 64-bit states.
- Support seeded and custom-secret XXH3, seeded CityHash, seeded MurmurHash3, custom FNV offset bases, and caller-owned or borrowed XXH3 secret storage without allocation.
- Accelerate long-input XXH3 with scalar, little-endian AArch64 NEON, x86-64 SSE2, and x86-64 AVX2 kernels selected from target features and cached runtime detection where available.
- Integrate streaming states with `std::io::Write` under the default `std` feature while keeping direct `update` and `digest` APIs available in every build.
