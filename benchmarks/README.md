# Benchmarks

The Divan benchmarks compare the public APIs of `rache`, `twox-hash`, and
`xxhash-rust`. They are development-only dependencies and are not linked into
the published library.

`throughput` covers XXH32, XXH64, XXH3-64, and XXH3-128 from 1 byte through
1 MiB. It includes the 240/241-byte XXH3 routing boundary and seeded XXH3
variants. `streaming` covers XXH3-64 and XXH3-128 with these input/chunk pairs:

- 4 KiB / 64 B
- 64 KiB / 1 KiB
- 1 MiB / 64 KiB

Run the complete suites:

```console
cargo bench -p benchmarks --bench throughput
cargo bench -p benchmarks --bench streaming
```

Divan accepts name filters, which are useful during optimization:

```console
cargo bench -p benchmarks --bench throughput -- xxh3_64
cargo bench -p benchmarks --bench throughput -- xxh3_64_seeded
cargo bench -p benchmarks --bench streaming -- rache
```
