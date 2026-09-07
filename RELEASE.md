# Releasing `hashcrew`

Only the `hashcrew` workspace package is published. Benchmarks, examples, and integration-test packages must remain development-only.

## Preflight

1. Prepare a release PR that updates the crate version, `Cargo.lock`, and `CHANGELOG.md`.
2. After the PR is merged, work from a clean, up-to-date checkout of `main`.
3. Run the release-comparison inputs in the [benchmark guide](benchmarks/README.md) when performance-relevant code changed.
4. Require every CI job to pass.

Select stable once for this release session. The commands below inherit it; `cargo x lint` selects nightly for Clippy and rustfmt, and `cargo +1.85.0 x test` checks the MSRV:

```shell
export RUSTUP_TOOLCHAIN=stable
cargo x lint
cargo x check
cargo x build --locked
cargo x test
cargo +1.85.0 x test
cargo release --package hashcrew
```

The `cargo release` command is a dry run unless `--execute` is present. Inspect `cargo package --package hashcrew --locked --list` before publishing. The archive must contain `LICENSE` with the applicable third-party terms, README, manifest, lockfile, and library sources, without workspace benchmarks or integration-test fixtures. The crate's `LICENSE` is a symbolic link to the root license. Update the root file and check its entries against the source-file attributions when incorporating or updating third-party code.

## Publish

Continue in the same shell with stable selected. Confirm the crate name, version, repository URL, and crates.io account before the irreversible step:

```shell
git switch main
git pull --ff-only origin main
cargo release --package hashcrew --execute
```

Omitting a version argument releases the stable version already recorded in the manifest. This publishes the crate, creates the configured signed `v<version>` tag, and pushes the branch and tag. Do not create a second tag manually. After crates.io confirms the release, verify the crate page and docs.rs build.
