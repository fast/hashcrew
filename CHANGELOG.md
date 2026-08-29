# Changelog

## 0.1.0

- Add compatible XXH32, XXH64, XXH3-64, and XXH3-128 implementations.
- Add CityHash32, CityHash64, CityHash128, seeded variants, and the public
  128-to-64 reducer as allocation-free one-shot APIs.
- Add MurmurHash3 x86_32 and x64_128 one-shot and streaming implementations.
- Add FNV-1a 32 and FNV-1a 64 one-shot and streaming implementations.
- Add reference-compatible XXH3 custom-secret and seed+secret raw, streaming,
  and `BuildHasher` APIs without allocation.
- Add FNV-1a custom offset-basis raw, streaming, and `BuildHasher` APIs.
- Add allocation-free raw and streaming APIs plus standard hashing adapters.
- Add scalar, Arm NEON, x86 SSE2, and x86 AVX2 XXH3 kernels.
- Optimize NEON stripe accumulation and instruction scheduling, plus buffered
  streaming writes that do not yet require kernel work.
- Add `no_std` support, randomized cross-implementation and per-backend tests,
  specification vectors, and one-shot/streaming comparison benchmarks.
- Add exhaustive short-input and two-way streaming partition tests, optimized
  CI coverage for MSRV and representative targets, and a reproducible package
  release gate.
- Adopt the Apache License 2.0 and enforce source headers and Conventional
  Commit pull-request titles in CI.
- Group implementations by algorithm family while preserving crate-root xxHash
  module paths.
