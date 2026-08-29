# rache

`rache` is a zero-dependency Rust library for fast, non-cryptographic hashing.
The first algorithm family is xxHash:

- XXH32 and XXH64
- XXH3-64 and XXH3-128
- allocation-free one-shot and streaming APIs
- `core::hash::Hasher` and deterministic `BuildHasher` adapters
- scalar, Arm NEON, x86 SSE2, and x86 AVX2 kernels for long XXH3 inputs
- `no_std` support with compile-time CPU feature selection

These algorithms are checksums and hash-table primitives. They are not suitable
for passwords, signatures, MACs, or untrusted-input denial-of-service defense.

## Quick start

```rust
use rache::{Xxh3, raw};

let one_shot = raw::xxh3_64(b"rache");

let mut streaming = Xxh3::new();
streaming.update(b"ra");
streaming.update(b"che");
assert_eq!(streaming.digest(), one_shot);
```

The raw functions take byte slices and return stable, platform-independent
integers. Prefer them when the complete input is already in memory.

## Workspace

Following the separation used by Apache Asyncband, the repository keeps the
publishable crate isolated from development-only dependencies:

```text
rache/              published library crate
benchmarks/         Divan one-shot and streaming comparisons
examples/           runnable programs
tests-integration/  cross-implementation conformance tests
```

Run the full validation suite with:

```console
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo check -p rache --no-default-features
```

## Benchmark snapshot

The benchmarks compare `rache` with `twox-hash` and `xxhash-rust` across short,
boundary, and large inputs. Both unseeded and seeded XXH3 one-shot APIs are
covered. These representative medians are a regression snapshot rather than a
performance guarantee:

| XXH3-64 case | rache | twox-hash | xxhash-rust |
|---|---:|---:|---:|
| one-shot, 4 KiB | 49.29 GB/s | 48.91 GB/s | 47.08 GB/s |
| one-shot, 1 MiB | 48.30 GB/s | 48.30 GB/s | 44.15 GB/s |
| seeded one-shot, 4 KiB | 48.89 GB/s | 48.89 GB/s | 46.71 GB/s |
| streaming, 1 MiB / 64 KiB chunks | 47.93 GB/s | 48.21 GB/s | 33.02 GB/s |

Reproduce the complete suites with:

```console
cargo bench -p benchmarks --bench throughput
cargo bench -p benchmarks --bench streaming
```

Focused runs accept Divan name filters:

```console
cargo bench -p benchmarks --bench throughput -- xxh3_64
cargo bench -p benchmarks --bench throughput -- xxh3_64_seeded
cargo bench -p benchmarks --bench streaming -- rache
```

See [`benchmarks/README.md`](benchmarks/README.md) for the complete case matrix.

## Kernel policy

Inputs up to 240 bytes use XXH3's specialized scalar paths. Longer inputs are
processed in 64-byte stripes. Features guaranteed by the target are selected
directly; other `std` builds cache runtime detection, while `no_std` uses only
compile-time features and otherwise falls back to scalar code. Inspect the
selected backend with `rache::kernel::selected_backend()`.

## Compatibility

Output compatibility is tested against the official xxHash behavior through an
independent Rust implementation at every algorithm boundary, with multiple
seeds and streaming chunk sizes. The minimum supported Rust version is 1.85.

## References

- [xxHash specification](https://github.com/Cyan4973/xxHash/blob/dev/doc/xxhash_spec.md)
- [twox-hash](https://github.com/shepmaster/twox-hash) for Rust API design ideas
- [Apache Asyncband](https://github.com/apache/asyncband) for workspace organization

Licensed under MIT. See `THIRD_PARTY_NOTICES.md` for implementation references.
