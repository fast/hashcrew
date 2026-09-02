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

use std::io::Cursor;
use std::io::Write;

use rache::fnv::Fnv1a32;
use rache::fnv::Fnv1a64;
use rache::murmur::Murmur3_32;
use rache::murmur::Murmur3_128;
use rache::murmur::murmur3_128;
use rache::xxhash::Xxh3;
use rache::xxhash::Xxh3_128;
use rache::xxhash::Xxh32;
use rache::xxhash::Xxh64;
use rache::xxhash::xxh3_64;
use rache::xxhash::xxh3_128;

#[test]
fn streaming_states_are_standard_io_writers() {
    fn assert_writer<T: Write>() {}

    assert_writer::<Fnv1a32>();
    assert_writer::<Fnv1a64>();
    assert_writer::<Murmur3_32>();
    assert_writer::<Murmur3_128>();
    assert_writer::<Xxh32>();
    assert_writer::<Xxh64>();
    assert_writer::<Xxh3>();
    assert_writer::<Xxh3_128>();

    let input = b"hash bytes read from a file or network stream";
    let mut hash = Xxh3::new();
    std::io::copy(&mut Cursor::new(input), &mut hash).unwrap();
    assert_eq!(hash.digest(), xxh3_64(input));

    let mut hash = Murmur3_128::new();
    assert_eq!(hash.write(input).unwrap(), input.len());
    assert_eq!(hash.digest(), murmur3_128(input, 0));

    let mut hash = Xxh3_128::new();
    assert_eq!(hash.write(input).unwrap(), input.len());
    assert_eq!(hash.digest(), xxh3_128(input));
}
