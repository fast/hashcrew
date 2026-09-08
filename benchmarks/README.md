# Benchmarks

The benchmark package uses Divan to measure public `hashcrew` APIs alongside `cityhasher`, `cityhash-rs`, `twox-hash`, `xxhash-rust`, `murmur3`, `fnv`, and RustCrypto's `md-5`. These comparison crates are development-only dependencies and are not linked into the published library.

| Target       | Coverage                                                                                                                                              |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `throughput` | Every supported one-shot variant from 0 bytes to 1 MiB, including seeded CityHash, seeded/custom-secret XXH3 APIs, and the XXH3 240/241-byte boundary |
| `streaming`  | Every streaming-capable variant with short messages, irregular chunks, and bulk input, including borrowed XXH3 custom secrets                         |

Both targets measure complete public-call paths and report byte throughput. The `murmur3` comparison therefore includes its `Read`-based interface rather than treating the compression core as a separate benchmark. CityHash is intentionally absent from `streaming`: a compatible incremental facade would have to retain the complete message.

Run both suites locally with `cargo x bench`, or select one:

```shell
cargo x bench throughput
cargo x bench streaming
```

Run the release-comparison input sizes:

```shell
cargo x bench throughput -- 4096 1048576 --min-time 0.1 --max-time 0.25
cargo x bench streaming -- 1048576 --min-time 0.1 --max-time 0.25
```

Divan name filters are useful while changing one family or API:

```shell
cargo x bench throughput -- xxh3_64
cargo x bench throughput -- xxh3_64_seeded
cargo x bench throughput -- xxh3_64_secret
cargo x bench throughput -- xxh3_64_seed_and_secret
cargo x bench throughput -- cityhash
cargo x bench throughput -- murmur3
cargo x bench throughput -- fnv1a
cargo x bench throughput -- md5
cargo x bench streaming -- md5
cargo x bench streaming -- xxh64
cargo x bench streaming -- hashcrew
```

The streaming cases include 32 B / 8 B chunks, 241 B / 17 B chunks, 4 KiB / 7 B chunks, 4 KiB / 64 B chunks, 4 KiB / 1 KiB chunks, 64 KiB / 1 KiB chunks, and 1 MiB / 64 KiB chunks. The repeated 4 KiB size isolates chunking overhead from total input size. Each iteration includes construction, all updates, and the final digest; it does not measure only the compression loop. Input generation happens outside the timed closure.

CI uses `cargo x build --locked` to compile all workspace targets, including both benchmark executables, without running measurements. Use the same command locally when only a build check is needed.

Pass harness options after `--`. For example, `cargo x bench streaming -- --list` lists cases and `cargo x bench streaming -- --help` shows Divan options.

Use a fixed toolchain and the same target features for all compared implementations. Run suites serially, repeat measurements, and compare medians; results from another CPU or compiler are not directly comparable. Constant seeds in the one-shot suite represent fixed-seed workloads and may permit compiler specialization. The suite compares Rust implementations, not the upstream C/C++ libraries. Some variants lack a comparable API in these dependencies, including seeded CityHash128, two-seed CityHash64, FNV-1a 32, and incremental MurmurHash3. Custom-secret streaming currently measures only `hashcrew`.
