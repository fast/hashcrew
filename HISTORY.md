# Implementation history

The first `rache` release establishes the xxHash family as the library's initial
compatibility surface. XXH32 and XXH64 use direct portable cores. XXH3 keeps
short-input routing separate from a long-input kernel so hardware backends can
change without changing public APIs or digest output.

The naming and API split draw on lessons from `twox-hash`: one-shot functions
for complete byte slices, stateful types for streaming input, and explicit
adapters for Rust's standard hashing traits. Unlike implementations whose
streaming XXH3 state allocates its secret, the initial `rache` state owns fixed
buffers so the same API works under `no_std`.

The repository separates the publishable crate from examples, benchmarks, and
cross-implementation tests, following the workspace organization used by
Apache Asyncband.
