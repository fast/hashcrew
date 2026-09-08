// Copyright 2026 FastLabs Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::fmt::Debug;
use std::io::Cursor;
use std::io::Write;

use hashcrew::fnv;
use hashcrew::md5;
use hashcrew::murmur;
use hashcrew::xxhash;

fn assert_writer<W: Write, D: Debug + Eq>(
    mut state: W,
    digest: impl Fn(&W) -> D,
    expected: impl Fn(&[u8]) -> D,
) {
    let input: Vec<_> = (0..17_003).map(|index| (index * 131 + 17) as u8).collect();
    assert_eq!(state.write(&[]).unwrap(), 0);
    state.flush().unwrap();
    assert_eq!(digest(&state), expected(&[]));

    assert_eq!(state.write(&input[..17]).unwrap(), 17);
    assert_eq!(digest(&state), expected(&input[..17]));
    state.flush().unwrap();
    assert_eq!(state.write(&[]).unwrap(), 0);
    assert_eq!(digest(&state), expected(&input[..17]));

    state.write_all(&input[17..63]).unwrap();
    let copied = std::io::copy(&mut Cursor::new(&input[63..]), &mut state).unwrap();
    assert_eq!(copied, (input.len() - 63) as u64);
    state.flush().unwrap();
    assert_eq!(digest(&state), expected(&input));
}

#[test]
fn streaming_states_are_standard_io_writers() {
    assert_writer(md5::Md5::new(), md5::Md5::digest, md5::md5);
    assert_writer(fnv::Fnv1a32::new(), fnv::Fnv1a32::digest, fnv::fnv1a_32);
    assert_writer(fnv::Fnv1a64::new(), fnv::Fnv1a64::digest, fnv::fnv1a_64);
    assert_writer(
        murmur::Murmur3X86_32::with_seed(42),
        murmur::Murmur3X86_32::digest,
        |bytes| murmur::murmur3_x86_32(bytes, 42),
    );
    assert_writer(
        murmur::Murmur3X86_128::with_seed(42),
        murmur::Murmur3X86_128::digest,
        |bytes| murmur::murmur3_x86_128(bytes, 42),
    );
    assert_writer(
        murmur::Murmur3X64_128::with_seed(42),
        murmur::Murmur3X64_128::digest,
        |bytes| murmur::murmur3_x64_128(bytes, 42),
    );
    assert_writer(
        xxhash::Xxh32::with_seed(42),
        xxhash::Xxh32::digest,
        |bytes| xxhash::xxh32(bytes, 42),
    );
    assert_writer(
        xxhash::Xxh64::with_seed(42),
        xxhash::Xxh64::digest,
        |bytes| xxhash::xxh64(bytes, 42),
    );
    assert_writer(
        xxhash::Xxh3_64::with_seed(42),
        xxhash::Xxh3_64::digest,
        |bytes| xxhash::xxh3_64_with_seed(bytes, 42),
    );
    assert_writer(
        xxhash::Xxh3_128::with_seed(42),
        xxhash::Xxh3_128::digest,
        |bytes| xxhash::xxh3_128_with_seed(bytes, 42),
    );
}
