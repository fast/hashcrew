# Benchmarks

The benchmark package uses Divan to measure public `hashcrew` APIs alongside `cityhasher`, `cityhash-rs`, `crc`, `crc32fast`, `crc32c`, `twox-hash`, `xxhash-rust`, `murmur3`, `fnv`, `crc-fast`, and RustCrypto's `md-5`. These comparison crates are development-only dependencies and are not linked into the published library.

| Target       | Coverage                                                                                                                                              |
| ------------ | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `throughput` | Every supported one-shot variant from 0 bytes to 1 MiB, including seeded CityHash, seeded/custom-secret XXH3 APIs, and the XXH3 240/241-byte boundary |
| `streaming`  | Every streaming-capable variant with short messages, irregular chunks, and bulk input, including borrowed XXH3 custom secrets                         |

Both targets measure complete public-call paths and report byte throughput. Hashcrew enables `std` for runtime CPU detection; `no_std` callers need suitable target features to enable hardware kernels. The `murmur3` comparison therefore includes its `Read`-based interface rather than treating the compression core as a separate benchmark. CityHash is intentionally absent from `streaming`: a compatible incremental facade would have to retain the complete message.

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
cargo x bench throughput -- crc32
cargo x bench streaming -- crc32
cargo x bench streaming -- xxh64
cargo x bench streaming -- hashcrew
```

The streaming cases include 32 B / 8 B chunks, 241 B / 17 B chunks, 4 KiB / 7 B chunks, 4 KiB / 64 B chunks, 4 KiB / 1 KiB chunks, 64 KiB / 1 KiB chunks, and 1 MiB / 64 KiB chunks. The repeated 4 KiB size isolates chunking overhead from total input size. Each iteration includes construction, all updates, and the final digest; it does not measure only the compression loop. Input generation happens outside the timed closure.

CRC comparisons keep the algorithm fixed: `crc32_iso_hdlc` compares IEEE CRC32 against `crc32fast`, and `crc32_iscsi` compares Castagnoli CRC32C against `crc32c`. Both include the default `crc::Table<1>` and `crc::Table<16>` implementations, with tables prepared outside timing. The CRC size matrix includes hardware-path boundaries and a separate one-byte-offset comparison for unaligned input. Each library selects its available hardware acceleration. Hashcrew uses AArch64 CRC/PMULL and x86-64 CRC32/PCLMULQDQ; x86-64 AVX2/AVX-512 VPCLMULQDQ also requires Rust 1.89 or newer. The `crc32c` streaming case uses its finalized-checksum append API.

Both CRC variants also include `crc-fast` 1.10. The throughput target measures its specialized helper and generic algorithm-selector API; the streaming target measures its `Digest`. Every comparison is enabled by default, and the benchmark package requires Rust 1.89 or newer. Hashcrew itself retains its Rust 1.85 minimum.

Performance regressions are checked locally. Compare the base and candidate revisions on the same machine with the same toolchain, target features, inputs, and sampling settings. Alternate their execution order across repeated runs, and keep the complete measurements together with the commit IDs and environment details.

For local CRC measurements, fixed sample sizes avoid automatic calibration settling on a single call near the timer resolution after a slow initial sample. Use `--sample-size 1024` for inputs through the 16 KiB boundary cases and `--sample-size 16` for 64 KiB / 1 MiB inputs, keeping the batch size identical across compared implementations. Longer 1 MiB comparisons can use 64-call samples to check whether an apparent difference persists.

CI only checks that benchmarks compile; it does not run performance measurements or enforce timing thresholds. Stable CI uses `cargo x build --locked` to compile all workspace targets with every Hashcrew family and comparison enabled, including both benchmark executables. Use the same command with Rust 1.89 or newer when only a build check is needed. `cargo x test` excludes the benchmark package so library and integration tests also run on Hashcrew's Rust 1.85 minimum.

Pass harness options after `--`. For example, `cargo x bench streaming -- --list` lists cases and `cargo x bench streaming -- --help` shows Divan options.

Use a fixed toolchain and the same target features for all compared implementations. Run suites serially, repeat measurements, and compare medians; results from another CPU or compiler are not directly comparable. Constant seeds in the one-shot suite represent fixed-seed workloads and may permit compiler specialization. The suite compares Rust implementations, not the upstream C/C++ libraries. Some variants lack a comparable API in these dependencies, including seeded CityHash128, two-seed CityHash64, FNV-1a 32, and incremental MurmurHash3. Custom-secret streaming currently measures only `hashcrew`.
