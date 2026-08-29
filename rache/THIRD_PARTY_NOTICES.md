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

The MurmurHash3 constants and compatibility behavior are based on Austin
Appleby's reference implementation:

- <https://github.com/aappleby/smhasher/blob/master/src/MurmurHash3.cpp>

The FNV-1a constants and compatibility vectors are based on RFC 9923:

- <https://www.rfc-editor.org/rfc/rfc9923.html>

The CityHash algorithms, constants, and compatibility behavior are based on
Google's CityHash 1.1.1 reference implementation, distributed under the MIT
License:

- <https://github.com/google/cityhash>

`xxhash-rust` is a development-only conformance and benchmark dependency. It is
not linked into the published `rache` crate:

- <https://github.com/DoumanAsh/xxhash-rust>

`murmur3` and `fnv` are development-only conformance and benchmark
dependencies. They are not linked into the published `rache` crate:

- <https://github.com/stusmall/murmur3>
- <https://github.com/servo/rust-fnv>

`cityhasher` and `cityhash-rs` are development-only conformance and benchmark
dependencies. They are not linked into the published `rache` crate:

- <https://github.com/khonsulabs/cityhasher>
- <https://github.com/Protryon/cityhash-rs>
