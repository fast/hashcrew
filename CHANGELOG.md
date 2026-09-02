# Changelog

## 0.2.0

- Remove duplicate crate-root, `raw`, and nested xxHash variant paths so each public API has one family-qualified home. Import functions, states, builders, constants, and kernels from `rache::{cityhash, fnv, murmur, xxhash}`; for example, replace `rache::Xxh3` or `rache::xxhash::xxh3::Xxh3` with `rache::xxhash::Xxh3`, `rache::raw::xxh3_64` with `rache::xxhash::xxh3_64`, and `rache::kernel` with `rache::xxhash::kernel`.
- Remove redundant alternate entry points so each operation has one public name. Use the module-level functions for complete input, replace `murmur3_128` and `Murmur3_128` with the architecture-qualified `murmur3_x64_128` and `Murmur3X64_128`, replace `Xxh3_64` with `Xxh3`, and replace the 128-bit states' `finish_128` methods with `digest`; retired alternate names remain searchable as rustdoc aliases where applicable.
- Allow custom-secret XXH3 streaming states and builders to own caller-provided storage while preserving borrowed-secret construction and allocation-free hashing. Code that explicitly named `Xxh3SecretBuilder<'a>` must use the storage type `Xxh3SecretBuilder<&'a [u8]>`; constructor calls with inferred types continue to work.
- Implement `std::io::Write` for every streaming hash state when the default `std` feature is enabled, allowing file and network hashing through `std::io::copy`, `Write::write`, or `Write::write_all`. The inherent 128-bit state `write` aliases have been removed so they do not shadow the trait method; use `update` for direct incremental input.
- Cache the derived secret in seeded `Xxh3Builder` values so hash collections do not repeat 192-byte seed expansion for every key.
- Improve one-shot XXH3-64 and XXH3-128 throughput for 17-to-128-byte inputs by ensuring the specialized medium-length paths are inlined.

## 0.1.0

- Add compatible XXH32, XXH64, XXH3-64, and XXH3-128 implementations.
- Add CityHash32, CityHash64, CityHash128, seeded variants, and the public 128-to-64 reducer as allocation-free one-shot APIs.
- Add MurmurHash3 x86_32 and x64_128 one-shot and streaming implementations.
- Add FNV-1a 32 and FNV-1a 64 one-shot and streaming implementations.
- Add reference-compatible XXH3 custom-secret and seed+secret raw, streaming, and `BuildHasher` APIs without allocation.
- Add FNV-1a custom offset-basis raw, streaming, and `BuildHasher` APIs.
- Add allocation-free raw and streaming APIs plus standard hashing adapters.
- Add scalar, little-endian AArch64 NEON, x86-64 SSE2, and x86-64 AVX2 XXH3 kernels.
- Optimize NEON stripe accumulation and instruction scheduling, plus buffered streaming writes that do not yet require kernel work.
- Add `no_std` support, randomized cross-implementation and per-backend tests, specification vectors, and one-shot/streaming comparison benchmarks.
- Add exhaustive short-input and two-way streaming partition tests, optimized CI coverage for MSRV and representative targets, and a reproducible package release gate.
- Harden XXH streaming state across 64-bit length-counter wraparound and avoid secret-size arithmetic overflow in XXH3 long-input one-shot hashing.
- Clarify that cross-platform digest stability applies to identical raw byte streams rather than Rust's native typed `Hash` encoding.
- Adopt the Apache License 2.0 and enforce source headers and Conventional Commit pull-request titles in CI.
- Group implementations by algorithm family while preserving crate-root xxHash module paths.
