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

use std::env;
use std::fs::File;
use std::io;

use rache::xxhash::Xxh3;

fn main() -> io::Result<()> {
    let path = env::args_os()
        .nth(1)
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "usage: hash-file <path>"))?;
    let mut file = File::open(&path)?;
    let mut hasher = Xxh3::new();
    io::copy(&mut file, &mut hasher)?;

    println!("{:016x}  {}", hasher.digest(), path.to_string_lossy());
    Ok(())
}
