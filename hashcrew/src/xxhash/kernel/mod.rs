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

//! XXH3 execution backends.
//!
//! Short XXH3 inputs use their dedicated scalar algorithms. For inputs larger
//! than 240 bytes, `hashcrew` directly selects features guaranteed by the target.
//! Otherwise, a `std` build caches runtime feature detection and a `no_std`
//! build uses target features, falling back to the portable scalar kernel.

mod scalar;

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
mod neon;
#[cfg(target_arch = "x86_64")]
mod x86;

#[cfg(all(target_arch = "aarch64", target_endian = "little"))]
pub(crate) use neon::Neon;
pub(crate) use scalar::Scalar;
#[cfg(target_arch = "x86_64")]
pub(crate) use x86::Avx2;
#[cfg(target_arch = "x86_64")]
pub(crate) use x86::Sse2;

/// A backend used by the XXH3 long-input kernel.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Backend {
    /// Portable scalar arithmetic.
    Scalar,
    /// 128-bit AArch64 NEON.
    Neon,
    /// 128-bit x86-64 SSE2.
    Sse2,
    /// 256-bit x86-64 AVX2.
    Avx2,
}

impl Backend {
    /// Returns whether this backend can execute safely on the current CPU.
    #[must_use]
    pub fn is_available(self) -> bool {
        match self {
            Self::Scalar => true,
            #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
            Self::Neon => {
                #[cfg(feature = "std")]
                {
                    std::arch::is_aarch64_feature_detected!("neon")
                }
                #[cfg(not(feature = "std"))]
                {
                    cfg!(target_feature = "neon")
                }
            }
            #[cfg(target_arch = "x86_64")]
            Self::Sse2 => {
                #[cfg(feature = "std")]
                {
                    std::arch::is_x86_feature_detected!("sse2")
                }
                #[cfg(not(feature = "std"))]
                {
                    cfg!(target_feature = "sse2")
                }
            }
            #[cfg(target_arch = "x86_64")]
            Self::Avx2 => {
                #[cfg(feature = "std")]
                {
                    std::arch::is_x86_feature_detected!("avx2")
                }
                #[cfg(not(feature = "std"))]
                {
                    cfg!(target_feature = "avx2")
                }
            }
            _ => false,
        }
    }
}

/// Returns the backend automatically selected for XXH3 long inputs.
#[must_use]
pub fn selected_backend() -> Backend {
    if cfg!(all(
        target_arch = "aarch64",
        target_endian = "little",
        target_feature = "neon"
    )) {
        Backend::Neon
    } else if cfg!(all(target_arch = "x86_64", target_feature = "avx2")) {
        Backend::Avx2
    } else {
        #[cfg(feature = "std")]
        {
            use std::sync::OnceLock;

            static SELECTED: OnceLock<Backend> = OnceLock::new();
            *SELECTED.get_or_init(detect_backend)
        }
        #[cfg(not(feature = "std"))]
        {
            detect_backend()
        }
    }
}

fn detect_backend() -> Backend {
    if Backend::Avx2.is_available() {
        Backend::Avx2
    } else if Backend::Sse2.is_available() {
        Backend::Sse2
    } else if Backend::Neon.is_available() {
        Backend::Neon
    } else {
        Backend::Scalar
    }
}

pub(crate) trait Xxh3Kernel: Copy {
    fn accumulate(self, acc: &mut [u64; 8], stripe: &[u8; 64], secret: &[u8; 64]);

    fn scramble(self, acc: &mut [u64; 8], secret: &[u8; 64]);
}

macro_rules! dispatch {
    ($function:ident($($argument:expr),* $(,)?)) => {{
        match $crate::xxhash::kernel::selected_backend() {
            $crate::xxhash::kernel::Backend::Scalar => {
                $function($crate::xxhash::kernel::Scalar, $($argument),*)
            }
            #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
            $crate::xxhash::kernel::Backend::Neon => {
                // SAFETY: Backend selection checked the current CPU for NEON.
                $function(unsafe { $crate::xxhash::kernel::Neon::new_unchecked() }, $($argument),*)
            }
            #[cfg(target_arch = "x86_64")]
            $crate::xxhash::kernel::Backend::Sse2 => {
                // SAFETY: Backend selection checked the current CPU for SSE2.
                $function(unsafe { $crate::xxhash::kernel::Sse2::new_unchecked() }, $($argument),*)
            }
            #[cfg(target_arch = "x86_64")]
            $crate::xxhash::kernel::Backend::Avx2 => {
                // SAFETY: Backend selection checked the current CPU for AVX2.
                $function(unsafe { $crate::xxhash::kernel::Avx2::new_unchecked() }, $($argument),*)
            }
            _ => unreachable!("backend cannot be selected on this target"),
        }
    }};
}

pub(crate) use dispatch;
