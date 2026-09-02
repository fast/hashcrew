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

//! xxHash one-shot, streaming, hash-table, and XXH3 kernel APIs.
//!
//! Use [`xxh32`] and [`Xxh32`] for XXH32, [`xxh64`] and [`Xxh64`] for XXH64,
//! and [`xxh3_64`]/[`Xxh3_64`] or [`xxh3_128`]/[`Xxh3_128`] for XXH3. Every
//! variant supports complete byte slices and bounded-memory incremental input.
//! The 32- and 64-bit states implement [`core::hash::Hasher`] and have matching
//! builders; the 128-bit state keeps its full `u128` result instead.
//!
//! XXH3 is the preferred general-purpose family for new trusted-input use. Its
//! explicitly named configuration functions and constructors support a seed, a
//! custom secret, or the reference algorithm's seed-and-secret routing. Custom
//! secrets must contain at least [`SECRET_SIZE_MIN`] bytes. They change the
//! deterministic output but do not protect attacker-controlled hash tables.
//!
//! With the default `std` feature, every streaming state implements
//! [`std::io::Write`]. The [`kernel`] module reports and exposes the selected
//! scalar or hardware XXH3 backend; ordinary callers do not need to choose one.
//!
//! ```
//! use hashcrew::xxhash::Xxh3_64;
//! use hashcrew::xxhash::xxh3_64;
//!
//! let mut state = Xxh3_64::new();
//! state.update(b"hash");
//! state.update(b"crew");
//! assert_eq!(state.digest(), xxh3_64(b"hashcrew"));
//! ```

mod xxh3;
mod xxh32;
mod xxh64;

pub mod kernel;

pub use self::xxh3::DEFAULT_SECRET;
pub use self::xxh3::DEFAULT_SECRET_SIZE;
pub use self::xxh3::SECRET_SIZE_MIN;
pub use self::xxh3::Xxh3_64;
pub use self::xxh3::Xxh3_64Builder;
pub use self::xxh3::Xxh3_64SecretBuilder;
pub use self::xxh3::Xxh3_128;
pub use self::xxh3::Xxh3SecretTooShort;
pub use self::xxh3::xxh3_64;
pub use self::xxh3::xxh3_64_with_secret;
pub use self::xxh3::xxh3_64_with_seed;
pub use self::xxh3::xxh3_64_with_seed_and_secret;
pub use self::xxh3::xxh3_128;
pub use self::xxh3::xxh3_128_with_secret;
pub use self::xxh3::xxh3_128_with_seed;
pub use self::xxh3::xxh3_128_with_seed_and_secret;
pub use self::xxh32::Xxh32;
pub use self::xxh32::Xxh32Builder;
pub use self::xxh32::xxh32;
pub use self::xxh64::Xxh64;
pub use self::xxh64::Xxh64Builder;
pub use self::xxh64::xxh64;
