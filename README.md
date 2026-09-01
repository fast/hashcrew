# rache

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]
[![MSRV 1.85][msrv-badge]](https://www.whatrustisit.com)
[![Apache 2.0 licensed][license-badge]][license-url]
[![Build Status][actions-badge]][actions-url]

[crates-badge]: https://img.shields.io/crates/v/rache.svg
[crates-url]: https://crates.io/crates/rache
[docs-badge]: https://docs.rs/rache/badge.svg
[docs-url]: https://docs.rs/rache
[msrv-badge]: https://img.shields.io/badge/MSRV-1.85-green?logo=rust
[license-badge]: https://img.shields.io/crates/l/rache
[license-url]: LICENSE
[actions-badge]: https://github.com/fast/rache/actions/workflows/ci.yml/badge.svg
[actions-url]: https://github.com/fast/rache/actions/workflows/ci.yml

## Overview

Rache is a zero-dependency Rust library for fast, deterministic, non-cryptographic hashing. It provides allocation-free one-shot APIs, incremental state where the algorithm supports it, stable cross-platform digests for identical raw byte streams, and hardware-accelerated XXH3 kernels.

Every implementation supports `no_std`. XXH3 inputs longer than 240 bytes use a dedicated kernel layer with scalar, Arm NEON, x86 SSE2, and x86 AVX2 backends; the other algorithms use compact portable Rust cores.

> [!WARNING]
>
> These algorithms are not cryptographically secure. Deterministic hashers are also unsuitable for hash tables exposed to attacker-controlled keys because they do not protect against deliberate hash flooding.

## Getting started

```shell
cargo add rache
```

Use the `raw` module when the complete input is already in memory:

```rust
use rache::raw;

let data = b"rache";
let city = raw::cityhash64(data);
let xxh3 = raw::xxh3_64(data);
let murmur = raw::murmur3_128(data, 42);
let fnv = raw::fnv1a_64(data);

assert_ne!(city, 0);
assert_ne!(xxh3, 0);
assert_ne!(murmur, 0);
assert_ne!(fnv, 0);
```

Use a state type when data arrives incrementally:

```rust
use rache::{Murmur3_128, raw};

let mut hash = Murmur3_128::with_seed(42);
hash.update(b"ra");
hash.update(b"che");

assert_eq!(hash.digest(), raw::murmur3_128(b"rache", 42));
```

The main types and functions are re-exported at the crate root. Family-scoped paths such as `rache::xxhash::xxh3_64`, `rache::murmur::murmur3_32`, and `rache::cityhash::cityhash64` remain available when a qualified import is clearer.

## Algorithms

| Family      | Variants                            | Native configuration                      | Streaming state | `Hasher` / `BuildHasher` |
|-------------|-------------------------------------|-------------------------------------------|-----------------|--------------------------|
| CityHash    | CityHash32, CityHash64, CityHash128 | one/two 64-bit seeds and a 128-bit seed   | one-shot only   | —                        |
| xxHash      | XXH32, XXH64, XXH3-64, XXH3-128     | seeds; XXH3 custom secret and seed+secret | all variants    | XXH32, XXH64, XXH3-64    |
| MurmurHash3 | x86_32, x64_128                     | 32-bit seed                               | both variants   | x86_32                   |
| FNV-1a      | 32-bit, 64-bit                      | standard or custom offset basis           | both variants   | both variants            |

CityHash is intentionally one-shot. Its digest depends on the complete input length and tail, so a streaming facade would have to retain the entire message and would not provide bounded-memory incremental hashing.

XXH3 accepts custom secrets of at least 136 bytes and returns an error for shorter inputs. Its seed-and-secret APIs follow the reference contract: inputs up to 240 bytes use the seed, while longer inputs use the custom secret. Custom secrets and non-standard FNV offset bases alter deterministic output; neither makes these algorithms cryptographically secure.

## Portability

Raw and streaming digests are stable across platforms for identical byte streams. Rust's `Hash` and `BuildHasher` adapters use native typed encodings, including platform endianness and `usize` width, and are not a portable serialization format.

Target-guaranteed CPU features are selected at compile time. Other `std` builds cache runtime feature detection; `no_std` builds use compile-time features only and otherwise fall back to the scalar kernel. [`rache::kernel::selected_backend()`](https://docs.rs/rache/*/rache/kernel/fn.selected_backend.html) reports the selected XXH3 backend.

## Examples and benchmarks

Runnable examples live in the [`examples`](examples) workspace crate. The [`benchmarks`](benchmarks) crate contains one-shot and streaming comparisons with independent implementations; see its [benchmark guide](benchmarks/README.md) for filters, input sizes, and the complete case matrix.

Use the repository workflow commands to run them:

```console
cargo x test
cargo x bench
```

## Correctness

Integration tests compare CityHash, xxHash, and MurmurHash3 with independent implementations, and verify FNV-1a against RFC vectors plus an independent 64-bit implementation. The suite covers boundary lengths, multiple seeds, custom secrets, custom FNV offset bases, randomized inputs, streaming partitions, available hardware backends, and both `std` and `no_std` builds.

## Minimum Supported Rust Version (MSRV)

Rache's minimum supported rustc version is 1.85.0. The MSRV may be increased in a minor release.

## License and acknowledgements

This project is licensed under [Apache License, Version 2.0](LICENSE). See [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) for the specifications, implementations, and development-only comparison dependencies that informed Rache.
