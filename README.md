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

## API model

Rache exposes the same algorithm at different integration boundaries. Pick the narrowest interface that matches where the bytes come from:

| Input or caller                                      | Interface                                                                 | What it does                                                                                 |
|------------------------------------------------------|---------------------------------------------------------------------------|----------------------------------------------------------------------------------------------|
| One complete byte slice                             | A module-level function such as `xxh3_64(input)`                          | Computes and returns the digest immediately without constructing a state.                    |
| Byte slices arriving incrementally                  | A state such as `Xxh3`: construct, call `update`, then call `digest`       | Retains bounded working state; `digest` does not consume it, and `reset` reuses its configuration. |
| A file, socket, decoder, or another `std::io` source | The same state through `std::io::Write` with the default `std` feature    | Treats every written byte as input; finish the producer, then call `digest` separately.       |
| A Rust hash collection or generic `Hash` caller     | A state through `Hasher`, usually constructed by its matching builder     | Accepts Rust's typed `Hash` encoding and returns a `u64` from `Hasher::finish`.                |

Every streaming type also provides associated `oneshot` functions that delegate to the corresponding module-level functions. They are namespaced conveniences, not a different hashing mode.

`Hasher` only supports a `u64` result, so 128-bit states deliberately expose `digest() -> u128` instead of truncating their output. CityHash has neither a state nor standard adapters because it cannot hash incrementally with bounded memory.

## Algorithm and capability map

The table names the canonical module-level function for complete input. A trailing `*` means the family also provides explicitly named seeded, custom-secret, or custom-offset-basis forms.

| Variant             | Complete input          | Incremental state              | Digest | `Hasher` / `BuildHasher`                     |
|---------------------|-------------------------|--------------------------------|--------|----------------------------------------------|
| CityHash32          | `cityhash32`            | —                              | `u32`  | —                                            |
| CityHash64          | `cityhash64*`           | —                              | `u64`  | —                                            |
| CityHash128         | `cityhash128*`          | —                              | `u128` | —                                            |
| XXH32               | `xxh32`                 | `Xxh32`                        | `u32`  | `Xxh32` / `Xxh32Builder`                     |
| XXH64               | `xxh64`                 | `Xxh64`                        | `u64`  | `Xxh64` / `Xxh64Builder`                     |
| XXH3-64             | `xxh3_64*`              | `Xxh3` (`Xxh3_64` alias)       | `u64`  | `Xxh3` / `Xxh3Builder` or secret builder     |
| XXH3-128            | `xxh3_128*`             | `Xxh3_128`                     | `u128` | —                                            |
| MurmurHash3 x86_32  | `murmur3_32`            | `Murmur3_32`                   | `u32`  | `Murmur3_32` / `Murmur3_32Builder`           |
| MurmurHash3 x64_128 | `murmur3_x64_128`       | `Murmur3_128`                  | `u128` | —                                            |
| FNV-1a 32           | `fnv1a_32*`             | `Fnv1a32`                      | `u32`  | `Fnv1a32` / `Fnv1a32Builder`                 |
| FNV-1a 64           | `fnv1a_64*`             | `Fnv1a64`                      | `u64`  | `Fnv1a64` / `Fnv1a64Builder`                 |

The original MurmurHash3 family defines `x86_32`, `x86_128`, and `x64_128` variants. Rache implements `x86_32` as `murmur3_32` and `x64_128` as `murmur3_x64_128`; it does not implement `x86_128`. The shorter `murmur3_128` name is retained as an equivalent convenience API. `cityhash128_to_64` reduces an existing 128-bit CityHash value; it does not hash a new byte slice.

## Choosing an algorithm

Use XXH3 for a new general-purpose checksum, cache key, or trusted-input hash table unless interoperability requires another family. Choose a 128-bit result when the application hashes enough distinct values for 64-bit collision probability to matter. XXH32, XXH64, CityHash, MurmurHash3, and FNV-1a are primarily useful for matching an existing format, protocol, or data set; their different outputs are not interchangeable.

## Streaming input

Call `update` when the application already has byte slices, as in the getting-started example above. With the default `std` feature, every streaming state can also be used as the destination of `std::io::copy` or another producer that accepts `std::io::Write`.

The adapter treats every written byte as hash input; it accepts the complete buffer and has nothing to flush. It does not write the digest anywhere. Finish the producer first, then call `digest` on the state:

```rust
use std::io::{self, Cursor};
use rache::xxhash::Xxh3;
use rache::xxhash::xxh3_64;

let mut source = Cursor::new(b"rache");
let mut hash = Xxh3::new();
io::copy(&mut source, &mut hash).unwrap();

assert_eq!(hash.digest(), xxh3_64(b"rache"));
```

This adapter is only needed for `std` interoperability. The direct `update` API is available in both `std` and `no_std` builds.

## Hash tables

The 32-bit and 64-bit streaming states implement `core::hash::Hasher`, with matching `BuildHasher` types for trusted-input hash tables:

```rust
use std::collections::HashMap;
use rache::xxhash::Xxh3Builder;

let mut counts = HashMap::with_hasher(Xxh3Builder::with_seed(7));
counts.insert("rache", 1);

assert_eq!(counts["rache"], 1);
```

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
