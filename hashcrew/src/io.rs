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

//! Standard I/O adapters for streaming hash states.

macro_rules! impl_write {
    ($feature:literal, $($implementation:tt)*) => {
        #[cfg(feature = $feature)]
        $($implementation)* {
            #[inline]
            fn write(&mut self, input: &[u8]) -> std::io::Result<usize> {
                self.update(input);
                Ok(input.len())
            }

            #[inline]
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
    };
}

impl_write!("fnv", impl std::io::Write for crate::fnv::Fnv1a32);
impl_write!("fnv", impl std::io::Write for crate::fnv::Fnv1a64);
impl_write!("md5", impl std::io::Write for crate::md5::Md5);
impl_write!("murmur", impl std::io::Write for crate::murmur::Murmur3X86_32);
impl_write!("murmur", impl std::io::Write for crate::murmur::Murmur3X86_128);
impl_write!("murmur", impl std::io::Write for crate::murmur::Murmur3X64_128);
impl_write!("xxhash", impl std::io::Write for crate::xxhash::Xxh32);
impl_write!("xxhash", impl std::io::Write for crate::xxhash::Xxh64);
impl_write!("xxhash", impl<S: AsRef<[u8]>> std::io::Write for crate::xxhash::Xxh3_64<S>);
impl_write!("xxhash", impl<S: AsRef<[u8]>> std::io::Write for crate::xxhash::Xxh3_128<S>);
