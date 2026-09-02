# Architecture

`rache` separates its public API, portable algorithm cores, and optional hardware kernels:

```text
public API
└── cityhash | xxhash | murmur | fnv
    ├── one-shot functions
    ├── streaming states and standard adapters
    └── portable cores
        └── XXH3 kernel: scalar | NEON | SSE2 | AVX2
```

## Public API

One-shot functions accept complete byte slices and live beside their streaming types in the corresponding family module. Streaming types reuse the same compression routines and retain only the state needed for the next update. Outputs that fit in `u64` also have deterministic `Hasher` and `BuildHasher` adapters; 128-bit variants expose native `u128` digests instead of truncating them to satisfy `Hasher`.

Configuration follows the source algorithm rather than a synthetic common interface: xxHash and MurmurHash3 use seeds, CityHash exposes only its official seeded one-shot variants, and FNV-1a calls its initial state an offset basis. XXH3 additionally accepts custom secrets of any reference-compatible length and exposes the seed+secret short/long-input routing defined by xxHash.

CityHash remains a one-shot family. The reference algorithm incorporates the complete input length and reads its tail relative to the end of the message; bounded-memory state cannot derive the final digest from independently hashed chunks. `CityHash128` values store the reference high word in the most significant half and the low word in the least significant half.

The individual xxHash variants and hardware kernel are nested under `rache::xxhash`; no compatibility modules are re-exported at the crate root.

## Streaming state

MurmurHash3 retains at most one incomplete 4- or 16-byte block. XXH32 and XXH64 retain one incomplete stripe. Seeded XXH3 states own the derived 192-byte secret; custom-secret states own or borrow validated caller-provided storage. Both use the same fixed-size pending-input and accumulator buffers, so no streaming state allocates or replays the complete message. FNV-1a retains its configured offset basis so reset restores the same namespace and requires no tail buffer. CityHash has no streaming state because a correct facade would need to retain and replay the complete input.

## XXH3 kernel boundary

Only XXH3 stripe accumulation and accumulator scrambling vary by hardware. Input-length routing, secret derivation, final merging, XXH32, XXH64, CityHash, MurmurHash3, and FNV-1a remain portable Rust. Keeping SIMD behind this narrow boundary makes every backend directly comparable with the scalar kernel and limits unsafe code to the intrinsic implementations.

CPU features guaranteed by the compilation target are selected directly. Other `std` builds cache runtime feature detection once per process. Hardware kernels are available on little-endian AArch64 for NEON and x86-64 for SSE2 and AVX2; other architectures use the scalar kernel. A `no_std` build uses compile-time target features only and otherwise selects the scalar backend.

The AArch64 backend processes four 64-bit lanes per loop iteration. Its compiler-only scheduling barrier preserves independent NEON multiply-accumulate chains without reading memory or changing digest semantics.

## Verification boundary

Reference crates are confined to the non-publishable integration-test and benchmark packages. Integration tests compare every public variant with an independent implementation or published vectors across boundary lengths, seeds, custom-secret lengths, offset bases, randomized inputs, and streaming partitions. Hardware tests compare every backend available on the current CPU directly with the scalar kernel. CityHash is checked against separate 32/64-bit and 128-bit Rust implementations as well as published reference vectors.
