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
//! The family module mirrors the crate-root API while keeping the individual
//! variants and the XXH3 kernel layer together in the source tree.

pub mod kernel;
pub mod xxh3;
pub mod xxh32;
pub mod xxh64;

pub use xxh3::DEFAULT_SECRET;
pub use xxh3::DEFAULT_SECRET_SIZE;
pub use xxh3::SECRET_SIZE_MIN;
pub use xxh3::Xxh3;
pub use xxh3::Xxh3_64;
pub use xxh3::Xxh3_128;
pub use xxh3::Xxh3Builder;
pub use xxh3::Xxh3SecretBuilder;
pub use xxh3::Xxh3SecretTooShort;
pub use xxh3::xxh3_64;
pub use xxh3::xxh3_64_with_secret;
pub use xxh3::xxh3_64_with_seed;
pub use xxh3::xxh3_64_with_seed_and_secret;
pub use xxh3::xxh3_128;
pub use xxh3::xxh3_128_with_secret;
pub use xxh3::xxh3_128_with_seed;
pub use xxh3::xxh3_128_with_seed_and_secret;
pub use xxh32::Xxh32;
pub use xxh32::Xxh32Builder;
pub use xxh32::xxh32;
pub use xxh64::Xxh64;
pub use xxh64::Xxh64Builder;
pub use xxh64::xxh64;
