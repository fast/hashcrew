# Benchmarks

The benchmark package uses Divan to measure public `rache` APIs alongside
`cityhasher`, `cityhash-rs`, `twox-hash`, `xxhash-rust`, `murmur3`, and `fnv`.
These comparison crates are development-only dependencies and are not linked
into the published library.

| Target | Coverage |
|---|---|
| `throughput` | Every supported one-shot variant from 1 byte to 1 MiB, including seeded CityHash, seeded/custom-secret XXH3 APIs, and the XXH3 240/241-byte boundary |
| `streaming` | Every streaming-capable variant at 4 KiB / 64 B chunks, 64 KiB / 1 KiB chunks, and 1 MiB / 64 KiB chunks, including borrowed XXH3 custom secrets |

Both targets measure complete public-call paths and report byte throughput.
The `murmur3` comparison therefore includes its `Read`-based interface rather
than treating the compression core as a separate benchmark.
CityHash is intentionally absent from `streaming`: a compatible incremental
facade would have to retain the complete message.

Run the complete suites:

```console
cargo bench -p benchmarks --bench throughput
cargo bench -p benchmarks --bench streaming
```

Run the release-comparison input sizes:

```console
cargo bench -p benchmarks --bench throughput -- 4096 1048576 --min-time 0.1 --max-time 0.25
cargo bench -p benchmarks --bench streaming -- 1048576 --min-time 0.1 --max-time 0.25
```

Divan name filters are useful while changing one family or API:

```console
cargo bench -p benchmarks --bench throughput -- xxh3_64
cargo bench -p benchmarks --bench throughput -- xxh3_64_seeded
cargo bench -p benchmarks --bench throughput -- xxh3_64_secret
cargo bench -p benchmarks --bench throughput -- xxh3_64_seed_and_secret
cargo bench -p benchmarks --bench throughput -- cityhash
cargo bench -p benchmarks --bench throughput -- murmur3
cargo bench -p benchmarks --bench throughput -- fnv1a
cargo bench -p benchmarks --bench streaming -- xxh64
cargo bench -p benchmarks --bench streaming -- rache
```
