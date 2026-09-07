# Releasing `hashcrew`

Only the `hashcrew` workspace package is published. Benchmarks, examples, and integration-test packages must remain development-only.

## Preflight

1. Work from a clean commit on the default branch.
2. Update the crate version and `CHANGELOG.md`.
3. Run the release-comparison inputs in the [benchmark guide](benchmarks/README.md) when performance-relevant code changed.
4. Require every CI job to pass.

Run the local release gate:

```shell
cargo x lint
cargo x check
cargo x test
RUSTUP_TOOLCHAIN=1.85.0 cargo x test
cargo x bench --no-run
release_version=0.1.0
cargo release "$release_version" --package hashcrew
```

The `cargo release` command is a dry run unless `--execute` is present. Inspect `cargo +stable package -p hashcrew --locked --list` before publishing. The archive must contain `LICENSE` with the applicable third-party terms, README, manifest, lockfile, and library sources, without workspace benchmarks or integration-test fixtures. The crate's `LICENSE` is a symbolic link to the root license. Update the root file and check its entries against the source-file attributions when incorporating or updating third-party code.

## Publish

Confirm the crate name, version, repository URL, and crates.io account before the irreversible step:

```shell
release_version=0.1.0
cargo release "$release_version" --package hashcrew --execute
```

This publishes the crate, creates the configured signed `v<version>` tag, and pushes the branch and tag. Do not create a second tag manually. After crates.io confirms the release, verify the crate page and docs.rs build.
