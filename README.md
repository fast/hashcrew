# rache

`rache` is a zero-dependency Rust library for fast, deterministic,
non-cryptographic hashing. It provides allocation-free one-shot APIs,
incremental state where the algorithm supports it, stable cross-platform
digests, and hardware-accelerated XXH3 kernels.

## Algorithms

| Family | Variants | Streaming state | `Hasher` / `BuildHasher` |
|---|---|---|---|
| CityHash | CityHash32, CityHash64, CityHash128, seeded variants | one-shot only | — |
| xxHash | XXH32, XXH64, XXH3-64, XXH3-128 | all variants | XXH32, XXH64, XXH3-64 |
| MurmurHash3 | x86_32, x64_128 | both variants | x86_32 |
| FNV-1a | 32-bit, 64-bit | both variants | both variants |

XXH3 inputs longer than 240 bytes use a dedicated kernel layer with scalar,
Arm NEON, x86 SSE2, and x86 AVX2 backends. The other algorithms use compact
portable Rust cores. Every implementation supports `no_std`.

CityHash is intentionally one-shot. Its digest depends on the complete input
length and tail, so a streaming facade would have to retain the entire message
and would not provide bounded-memory incremental hashing.

> These algorithms are not cryptographically secure. Deterministic hashers are
> also unsuitable for hash tables exposed to attacker-controlled keys because
> they do not protect against deliberate hash flooding.

## Usage

Use `raw` when the complete input is already in memory:

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

Use a streaming state when data arrives in chunks:

```rust
use rache::{Murmur3_128, raw};

let mut hash = Murmur3_128::with_seed(42);
hash.update(b"ra");
hash.update(b"che");

assert_eq!(hash.digest(), raw::murmur3_128(b"rache", 42));
```

The main types and functions are re-exported at the crate root. Family-scoped
paths such as `rache::xxhash::xxh3_64`, `rache::murmur::murmur3_32`, and
`rache::cityhash::cityhash64` are available when a qualified import is clearer.

## Repository layout

The publishable crate is isolated from all development-only dependencies:

```text
.
├── rache/                 publishable library crate
│   └── src/
│       ├── cityhash/       CityHash portable one-shot core
│       ├── xxhash/        XXH32, XXH64, XXH3, and hardware kernels
│       ├── murmur/        MurmurHash3 portable core and state
│       ├── fnv/           FNV-1a portable core and state
│       └── lib.rs         public exports and raw API
├── benchmarks/            Divan one-shot and streaming comparisons
├── examples/              runnable programs
└── tests-integration/     specification and cross-implementation tests
```

This workspace separation follows the same broad pattern as Apache Asyncband:
the `rache` package remains dependency-free while benchmarks and conformance
tests can use independent reference crates.

## Benchmark snapshot

The table records median throughput from the public APIs. One-shot cases use
4 KiB and 1 MiB inputs; streaming cases hash 1 MiB in 64 KiB chunks. Values are
a regression snapshot, not a performance guarantee.

| Algorithm | One-shot 4 KiB | One-shot 1 MiB | Streaming 1 MiB | Reference one-shot 1 MiB |
|---|---:|---:|---:|---:|
| CityHash32 | 9.37 GB/s | 9.27 GB/s | — | 9.16 GB/s (`cityhasher`) |
| CityHash64 | 28.64 GB/s | 28.53 GB/s | — | 28.50 GB/s (`cityhasher`) |
| CityHash128 | 28.13 GB/s | 28.66 GB/s | — | 28.66 GB/s (`cityhash-rs`) |
| XXH32 | 8.03 GB/s | 6.56 GB/s | 6.56 GB/s | 6.56 GB/s (`xxhash-rust`) |
| XXH64 | 27.39 GB/s | 27.44 GB/s | 27.68 GB/s | 27.80 GB/s (`twox-hash`) |
| XXH3-64 | 49.30 GB/s | 48.30 GB/s | 48.02 GB/s | 48.30 GB/s (`twox-hash`) |
| XXH3-128 | 49.29 GB/s | 48.30 GB/s | 47.93 GB/s | 48.30 GB/s (`twox-hash`) |
| MurmurHash3 x86_32 | 3.64 GB/s | 3.59 GB/s | 3.59 GB/s | 1.58 GB/s (`murmur3`) |
| MurmurHash3 x64_128 | 9.15 GB/s | 8.96 GB/s | 8.97 GB/s | 5.90 GB/s (`murmur3`) |
| FNV-1a 32 | 1.14 GB/s | 1.12 GB/s | 1.12 GB/s | — |
| FNV-1a 64 | 1.13 GB/s | 1.12 GB/s | 1.12 GB/s | 1.12 GB/s (`fnv`) |

The `murmur3` crate exposes a `Read`-based one-shot API, so those rows compare
complete public-call paths rather than isolated compression loops. Seeded
CityHash and XXH3 cases, plus additional input boundaries, are included in the
benchmark suite.

Reproduce this snapshot with:

```console
cargo bench -p benchmarks --bench throughput -- 4096 1048576 --min-time 0.1 --max-time 0.25
cargo bench -p benchmarks --bench streaming -- 1048576 --min-time 0.1 --max-time 0.25
```

Run every configured input size with:

```console
cargo bench -p benchmarks --bench throughput
cargo bench -p benchmarks --bench streaming
```

See [`benchmarks/README.md`](benchmarks/README.md) for filters and the complete
case matrix.

## Kernel selection

XXH3 routes inputs up to 240 bytes through specialized scalar paths and longer
inputs through 64-byte stripes. Target-guaranteed CPU features are selected at
compile time. Other `std` builds cache runtime feature detection; `no_std`
builds use compile-time features only and otherwise fall back to the scalar
kernel. `rache::kernel::selected_backend()` reports the selected backend.

## Correctness and compatibility

Integration tests compare CityHash, xxHash, and MurmurHash3 with independent
implementations, and verify FNV-1a against RFC vectors plus an independent
64-bit implementation. The suite covers boundary lengths, multiple seeds,
random inputs, every applicable streaming partition, every available hardware
backend, and both `std` and `no_std` builds.

The minimum supported Rust version is 1.85. Run the primary checks with:

```console
cargo +1.85.0 test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo +1.85.0 check -p rache --no-default-features
```

Maintainers should use the
[`RELEASING.md`](https://github.com/leiysky/rache/blob/main/RELEASING.md)
checklist for the complete package and publication gate.

## References

- [Google CityHash 1.1.1 reference implementation](https://github.com/google/cityhash)
- [xxHash specification](https://github.com/Cyan4973/xxHash/blob/dev/doc/xxhash_spec.md)
- [MurmurHash3 reference implementation](https://github.com/aappleby/smhasher/blob/master/src/MurmurHash3.cpp)
- [FNV specification (RFC 9923)](https://www.rfc-editor.org/rfc/rfc9923.html)
- [twox-hash](https://github.com/shepmaster/twox-hash) for Rust API design ideas
- [Apache Asyncband](https://github.com/apache/asyncband) for workspace organization

Licensed under the Apache License, Version 2.0. See
[`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) for implementation
references and development-only comparison dependencies.
