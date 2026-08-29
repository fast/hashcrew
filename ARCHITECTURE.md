# Architecture

The public API and algorithm machinery are deliberately separated:

```text
raw functions / streaming states / BuildHasher adapters
                         |
                  xxHash family cores
                         |
       scalar short paths + XXH3 long-input kernel
                         |
             scalar | NEON | SSE2 | AVX2
```

## API boundary

`rache::raw` is the preferred path for complete byte slices. Streaming types
retain enough trailing input to finalize XXH3 without allocation or replaying
the full message. Standard-library hashing traits are adapters over those same
states, not separate implementations.

## Kernel boundary

Only XXH3 stripe accumulation and accumulator scrambling vary by hardware.
Length routing, secret derivation, final merging, and all XXH32/XXH64 behavior
remain portable Rust. This keeps unsafe intrinsics small and makes every SIMD
backend directly comparable with the scalar kernel.

Features guaranteed by the compilation target are selected directly. For other
`std` builds, runtime feature detection is cached once per process. A `no_std`
build only selects features enabled for its compilation target, so it never
executes an intrinsic that the target contract does not provide.

The Arm kernel processes four 64-bit lanes per loop iteration. A compiler-only
scheduling barrier keeps the independent NEON multiply-accumulate chains from
being serialized; it does not access memory or change hash semantics.

## Correctness contract

Every backend must produce the same digest as the scalar kernel. Integration
tests additionally compare all four algorithms with an independent xxHash
implementation across boundary lengths, seeds, randomized inputs, and
streaming partitions. Every hardware kernel available on the test machine is
also compared directly with the scalar kernel.
