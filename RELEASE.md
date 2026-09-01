# Releasing `rache`

Only the `rache` workspace package is published. Benchmarks, examples, and integration-test packages must remain development-only.

## Preflight

1. Work from a clean commit on the default branch.
2. Update the crate version and `CHANGELOG.md`.
3. Refresh the README benchmark snapshot only when performance-relevant code changed.
4. Require every CI job to pass.

Run the local release gate:

```shell
cargo x lint
cargo x check
cargo x test
RUSTUP_TOOLCHAIN=1.85.0 cargo x test
cargo x bench --no-run
cargo +stable package -p rache --locked
cargo +stable publish -p rache --locked --dry-run
```

Inspect `cargo +stable package -p rache --locked --list` before publishing. The archive must contain the license, README, third-party notices, manifest, lockfile, and library sources, without workspace benchmarks or integration-test fixtures.

## Publish

Confirm the crate name, version, repository URL, and crates.io account before the irreversible step:

```shell
cargo +stable publish -p rache --locked
```

Create and push the matching `v<version>` tag only after crates.io confirms the release. Then verify the crate page and docs.rs build.
