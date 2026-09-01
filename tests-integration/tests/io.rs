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

use std::io::{Cursor, Write};

use rache::{Fnv1a32, Fnv1a64, Murmur3_32, Murmur3_128, Xxh3, Xxh3_128, Xxh32, Xxh64, xxh3_64};

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
}
