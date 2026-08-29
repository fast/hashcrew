# Third-party notices

The xxHash algorithms, constants, default secret, and compatibility behavior
are based on the public specification and reference implementation maintained
by Yann Collet and the xxHash contributors:

- <https://github.com/Cyan4973/xxHash>
- <https://github.com/Cyan4973/xxHash/blob/dev/doc/xxhash_spec.md>

The Rust API layering, streaming strategy, and SIMD implementation techniques
were informed by `twox-hash`, copyright (c) 2015 Jake Goulding, distributed
under the MIT License:

- <https://github.com/shepmaster/twox-hash>

The workspace organization was informed by Apache Asyncband, distributed under
the Apache License 2.0:

- <https://github.com/apache/asyncband>

`xxhash-rust` is a development-only conformance and benchmark dependency. It is
not linked into the published `rache` crate:

- <https://github.com/DoumanAsh/xxhash-rust>
