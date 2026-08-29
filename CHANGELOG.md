# Changelog

## 0.1.0

- Add compatible XXH32, XXH64, XXH3-64, and XXH3-128 implementations.
- Add allocation-free raw and streaming APIs plus standard hashing adapters.
- Add scalar, Arm NEON, x86 SSE2, and x86 AVX2 XXH3 kernels.
- Optimize NEON stripe accumulation and instruction scheduling, plus buffered
  streaming writes that do not yet require kernel work.
- Add `no_std` support, randomized cross-implementation and per-backend tests,
  examples, and one-shot/streaming comparison benchmarks.
