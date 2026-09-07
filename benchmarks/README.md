# Benchmarks

The benchmark package uses Divan to measure public `hashcrew` APIs alongside `cityhasher`, `cityhash-rs`, `twox-hash`, `xxhash-rust`, `murmur3`, and `fnv`. These comparison crates are development-only dependencies and are not linked into the published library.

| Target       | Coverage                                                                                                                                              |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `throughput` | Every supported one-shot variant from 0 bytes to 1 MiB, including seeded CityHash, seeded/custom-secret XXH3 APIs, and the XXH3 240/241-byte boundary |
| `streaming`  | Every streaming-capable variant with short messages, irregular chunks, and bulk input, including borrowed XXH3 custom secrets                         |

Both targets measure complete public-call paths and report byte throughput. The `murmur3` comparison therefore includes its `Read`-based interface rather than treating the compression core as a separate benchmark. CityHash is intentionally absent from `streaming`: a compatible incremental facade would have to retain the complete message.

Run the complete suites:

```shell
cargo x bench --bench throughput
cargo x bench --bench streaming
```

Run the release-comparison input sizes:

```shell
cargo x bench --bench throughput -- 4096 1048576 --min-time 0.1 --max-time 0.25
cargo x bench --bench streaming -- 1048576 --min-time 0.1 --max-time 0.25
```

Divan name filters are useful while changing one family or API:

```shell
cargo x bench --bench throughput -- xxh3_64
cargo x bench --bench throughput -- xxh3_64_seeded
cargo x bench --bench throughput -- xxh3_64_secret
cargo x bench --bench throughput -- xxh3_64_seed_and_secret
cargo x bench --bench throughput -- cityhash
cargo x bench --bench throughput -- murmur3
cargo x bench --bench throughput -- fnv1a
cargo x bench --bench streaming -- xxh64
cargo x bench --bench streaming -- hashcrew
```

The streaming cases include 32 B / 8 B chunks, 241 B / 17 B chunks, 4 KiB / 7 B chunks, 4 KiB / 64 B chunks, 4 KiB / 1 KiB chunks, 64 KiB / 1 KiB chunks, and 1 MiB / 64 KiB chunks. The repeated 4 KiB size isolates chunking overhead from total input size. Each iteration includes construction, all updates, and the final digest; it does not measure only the compression loop. Input generation happens outside the timed closure.

Pass harness options after `--`, or use `cargo x bench --no-run` to compile both targets without measuring. For example, `cargo x bench --bench streaming -- --list` lists cases and `cargo x bench --bench streaming -- --help` shows Divan options.

Use a fixed toolchain and the same target features for all compared implementations. Run suites serially, repeat measurements, and compare medians; results from another CPU or compiler are not directly comparable. Constant seeds in the one-shot suite represent fixed-seed workloads and may permit compiler specialization. The suite compares Rust implementations, not the upstream C/C++ libraries. Some variants lack a comparable API in these dependencies, including seeded CityHash128, two-seed CityHash64, FNV-1a 32, and incremental MurmurHash3. Custom-secret streaming currently measures only `hashcrew`.

See the [2026-09-07 review](2026-09-07-review.md) for a measured comparison, the XXH64 optimization, and remaining opportunities.
