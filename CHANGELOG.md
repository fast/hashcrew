# Changelog

All notable changes to this project will be documented in this file.

## Unreleased

## v0.2.0 (2026-09-08)

### Breaking changes

* Gate all hash families behind opt-in Cargo features and enable no features by default. Enable `cityhash`, `fnv`, `md5`, `murmur`, or `xxhash` for each family your application uses, and enable `std` for standard I/O adapters and XXH3 runtime CPU detection.

### New features

* Add MD5 behind the `md5` feature, with standard 16-byte digests, one-shot hashing, incremental updates, repeatable digest reads, and optional `std::io::Write` integration.

### Improvements

* Improve MurmurHash3 streaming throughput for short chunks on affected toolchains without link-time optimization.

## v0.1.2 (2026-09-07)

### Improvements

* Improve XXH64 streaming throughput for short chunks on affected toolchains.
* Preserve upstream copyright notices and license terms for incorporated hash implementations in source distributions, with third-party terms consolidated in `LICENSE`.

## v0.1.1 (2026-09-02)

### Improvements

* Improve seeded XXH3 one-shot latency for long inputs and XXH32 streaming throughput on affected AArch64 toolchains.

## v0.1.0 (2026-09-02)

This is the initial release. It provides dependency-free, allocation-free, and `no_std`-compatible implementations of the following hash families:

* CityHash32, CityHash64, and CityHash128.
* XXH32, XXH64, XXH3-64, and XXH3-128.
* MurmurHash3 x86_32, x86_128, and x64_128.
* FNV-1a 32 and 64.
