# Implementation history

Hashcrew starts with four algorithm families behind a consistent family-qualified API. One-shot functions handle complete byte slices, bounded-memory state types handle incremental input where the algorithm permits it, and explicit adapters integrate 32-bit and 64-bit states with Rust's standard hashing traits.

XXH32 and XXH64 use direct portable cores. XXH3 keeps short-input routing separate from a long-input kernel so scalar and hardware-accelerated backends can change without changing public APIs or digest output. Its streaming state owns fixed buffers and accepts borrowed or caller-owned secret storage, preserving allocation-free `no_std` operation.

MurmurHash3 implements the reference `x86_32`, `x86_128`, and `x64_128` variants. Their architecture labels identify distinct algorithms rather than target restrictions. FNV-1a 32 and 64 use a portable byte-at-a-time core shared by one-shot functions, streaming states, and standard hash adapters.

CityHash32, CityHash64, and CityHash128 remain one-shot-oriented. Their construction depends on the complete input length and tail, so a streaming facade would need to retain the entire message instead of providing bounded-memory incremental hashing.

The repository keeps the dependency-free publishable crate separate from examples, benchmarks, and cross-implementation tests. Reference implementations are development-only dependencies and never enter the published package.
