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
Apache Asyncband. Within the library, source files are grouped by algorithm
family; the XXH3 hardware kernels live beside the xxHash implementations while
crate-root re-exports keep the public API compact.

MurmurHash3 x86_32 and x64_128 extend the same raw/streaming split with small
fixed-size tail buffers. FNV-1a 32 and 64 use a single portable byte-at-a-time
core shared by their one-shot functions, streaming states, and standard hash
adapters. Reference crates remain isolated in the integration-test and
benchmark workspace members.

CityHash32, CityHash64, and CityHash128 add a second one-shot-oriented family,
including the reference seeded forms and 128-to-64 reducer. CityHash does not
expose incremental state: its length- and tail-dependent construction cannot
be resumed with bounded memory, so such an API would have to buffer the entire
message. The implementation stays portable, safe, allocation-free, and
compatible with `no_std`.
