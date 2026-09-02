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

//! xxHash family implementations and XXH3 hardware kernels.
//!
//! The family module keeps the individual variants and the XXH3 kernel layer
//! together under one public namespace.

mod xxh3;
mod xxh32;
mod xxh64;

pub mod kernel;

pub use self::xxh3::DEFAULT_SECRET;
pub use self::xxh3::DEFAULT_SECRET_SIZE;
pub use self::xxh3::SECRET_SIZE_MIN;
pub use self::xxh3::Xxh3;
pub use self::xxh3::Xxh3_64;
pub use self::xxh3::Xxh3_128;
pub use self::xxh3::Xxh3Builder;
pub use self::xxh3::Xxh3SecretBuilder;
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
