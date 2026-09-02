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

Every implementation supports `no_std`. XXH3 inputs longer than 240 bytes use a dedicated kernel layer with scalar, little-endian AArch64 NEON, x86-64 SSE2, and x86-64 AVX2 backends; the other algorithms use compact portable Rust cores.

> [!WARNING]
>
> These algorithms are not cryptographically secure. Deterministic hashers are also unsuitable for hash tables exposed to attacker-controlled keys because they do not protect against deliberate hash flooding.

## Getting started

```shell
cargo add rache
```

Disable the default `std` feature for bare-metal and other `no_std` targets:

```toml
[dependencies]
rache = { version = "0.2", default-features = false }
```

Import the algorithm family when the complete input is already in memory:

```rust
use rache::{cityhash, fnv, murmur, xxhash};

let data = b"rache";
let city = cityhash::cityhash64(data);
let xxh3 = xxhash::xxh3_64(data);
let murmur = murmur::murmur3_128(data, 42);
let fnv = fnv::fnv1a_64(data);

assert_ne!(city, 0);
assert_ne!(xxh3, 0);
assert_ne!(murmur, 0);
assert_ne!(fnv, 0);
```

Use a state type when data arrives incrementally:

```rust
use rache::murmur::Murmur3_128;
use rache::murmur::murmur3_128;

let mut hash = Murmur3_128::with_seed(42);
hash.update(b"ra");
hash.update(b"che");

assert_eq!(hash.digest(), murmur3_128(b"rache", 42));
```

Custom XXH3 secrets can be borrowed or moved into the streaming state. Owning the storage is useful when a factory or component needs to return a self-contained hasher:

```rust
use rache::xxhash::Xxh3;
use rache::xxhash::xxh3_64_with_secret;

let secret = [0xa5; 192];
let expected = xxh3_64_with_secret(b"rache", &secret).unwrap();
let mut hash = Xxh3::with_secret(secret).unwrap();
hash.update(b"rache");

assert_eq!(hash.digest(), expected);
```

All public APIs are grouped under the [`cityhash`](https://docs.rs/rache/*/rache/cityhash/), [`xxhash`](https://docs.rs/rache/*/rache/xxhash/), [`murmur`](https://docs.rs/rache/*/rache/murmur/), and [`fnv`](https://docs.rs/rache/*/rache/fnv/) modules. Each module keeps its one-shot functions, streaming states, builders, and configuration together.

## Choosing an algorithm

Use XXH3 for a new general-purpose checksum, cache key, or trusted-input hash table unless interoperability requires another family. Choose a 128-bit result when the application hashes enough distinct values for 64-bit collision probability to matter. XXH32, XXH64, CityHash, MurmurHash3, and FNV-1a are primarily useful for matching an existing format, protocol, or data set; their different outputs are not interchangeable.

## Standard adapters

With the default `std` feature, every streaming state implements [`std::io::Write`](https://doc.rust-lang.org/std/io/trait.Write.html), so files and network streams can be hashed with `std::io::copy`:

```rust
use std::io::{self, Cursor};
use rache::xxhash::Xxh3;
use rache::xxhash::xxh3_64;

let mut source = Cursor::new(b"rache");
let mut hash = Xxh3::new();
io::copy(&mut source, &mut hash).unwrap();

assert_eq!(hash.digest(), xxh3_64(b"rache"));
```

The 32-bit and 64-bit streaming states also implement `core::hash::Hasher`, with matching `BuildHasher` types for trusted-input hash tables:

```rust
use std::collections::HashMap;
use rache::xxhash::Xxh3Builder;

let mut counts = HashMap::with_hasher(Xxh3Builder::with_seed(7));
counts.insert("rache", 1);

assert_eq!(counts["rache"], 1);
```

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

Target-guaranteed CPU features are selected at compile time. Other `std` builds cache runtime feature detection; `no_std` builds use compile-time features only and otherwise fall back to the scalar kernel. [`rache::xxhash::kernel::selected_backend()`](https://docs.rs/rache/*/rache/xxhash/kernel/fn.selected_backend.html) reports the selected XXH3 backend.

## Examples and benchmarks

Runnable examples live in the [`examples`](examples) workspace crate. The [`benchmarks`](benchmarks) crate contains one-shot and streaming comparisons with independent implementations; see its [benchmark guide](benchmarks/README.md) for filters, input sizes, and the complete case matrix.

Use the repository workflow commands to run them:

```shell
cargo x test
cargo x bench
```

## Correctness

Integration tests compare CityHash, xxHash, and MurmurHash3 with independent implementations, and verify FNV-1a against RFC vectors plus an independent 64-bit implementation. The suite covers boundary lengths, multiple seeds, custom secrets, custom FNV offset bases, randomized inputs, streaming partitions, available hardware backends, and both `std` and `no_std` builds.

## Minimum Supported Rust Version (MSRV)

Rache's minimum supported rustc version is 1.85.0. The MSRV may be increased in a minor release.

## License and acknowledgements

This project is licensed under [Apache License, Version 2.0](LICENSE). See [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) for the specifications, implementations, and development-only comparison dependencies that informed Rache.
