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
//! build falls back to the portable scalar kernel.

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
            Self::Neon => neon_available(),
            Self::Sse2 => sse2_available(),
            Self::Avx2 => avx2_available(),
        }
    }
}

/// Returns the backend automatically selected for XXH3 long inputs.
#[must_use]
pub fn selected_backend() -> Backend {
    #[cfg(all(
        target_arch = "aarch64",
        target_endian = "little",
        target_feature = "neon"
    ))]
    {
        Backend::Neon
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))]
    {
        Backend::Avx2
    }

    #[cfg(not(any(
        all(
            target_arch = "aarch64",
            target_endian = "little",
            target_feature = "neon"
        ),
        all(target_arch = "x86_64", target_feature = "avx2")
    )))]
    selected_backend_fallback()
}

#[cfg(not(any(
    all(
        target_arch = "aarch64",
        target_endian = "little",
        target_feature = "neon"
    ),
    all(target_arch = "x86_64", target_feature = "avx2")
)))]
fn selected_backend_fallback() -> Backend {
    #[cfg(feature = "std")]
    {
        use std::sync::OnceLock;

        static SELECTED: OnceLock<Backend> = OnceLock::new();
        *SELECTED.get_or_init(detect_backend)
    }

    #[cfg(not(feature = "std"))]
    detect_backend()
}

#[cfg(not(any(
    all(
        target_arch = "aarch64",
        target_endian = "little",
        target_feature = "neon"
    ),
    all(target_arch = "x86_64", target_feature = "avx2")
)))]
fn detect_backend() -> Backend {
    if avx2_available() {
        Backend::Avx2
    } else if sse2_available() {
        Backend::Sse2
    } else if neon_available() {
        Backend::Neon
    } else {
        Backend::Scalar
    }
}

#[inline]
fn neon_available() -> bool {
    #[cfg(all(feature = "std", target_arch = "aarch64", target_endian = "little"))]
    {
        return std::arch::is_aarch64_feature_detected!("neon");
    }
    #[cfg(all(
        not(feature = "std"),
        target_arch = "aarch64",
        target_endian = "little"
    ))]
    {
        return cfg!(target_feature = "neon");
    }
    #[allow(unreachable_code)]
    false
}

#[inline]
fn sse2_available() -> bool {
    #[cfg(all(feature = "std", target_arch = "x86_64"))]
    {
        return std::arch::is_x86_feature_detected!("sse2");
    }
    #[cfg(all(not(feature = "std"), target_arch = "x86_64"))]
    {
        return cfg!(target_feature = "sse2");
    }
    #[allow(unreachable_code)]
    false
}

#[inline]
fn avx2_available() -> bool {
    #[cfg(all(feature = "std", target_arch = "x86_64"))]
    {
        return std::arch::is_x86_feature_detected!("avx2");
    }
    #[cfg(all(not(feature = "std"), target_arch = "x86_64"))]
    {
        return cfg!(target_feature = "avx2");
    }
    #[allow(unreachable_code)]
    false
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
            $crate::xxhash::kernel::Backend::Neon => {
                #[cfg(all(target_arch = "aarch64", target_endian = "little"))]
                {
                    // SAFETY: Backend selection checked the current CPU for NEON.
                    $function(unsafe { $crate::xxhash::kernel::Neon::new_unchecked() }, $($argument),*)
                }
                #[cfg(not(all(target_arch = "aarch64", target_endian = "little")))]
                unreachable!("NEON cannot be selected on this target")
            }
            $crate::xxhash::kernel::Backend::Sse2 => {
                #[cfg(target_arch = "x86_64")]
                {
                    // SAFETY: Backend selection checked the current CPU for SSE2.
                    $function(unsafe { $crate::xxhash::kernel::Sse2::new_unchecked() }, $($argument),*)
                }
                #[cfg(not(target_arch = "x86_64"))]
                unreachable!("SSE2 cannot be selected on this target")
            }
            $crate::xxhash::kernel::Backend::Avx2 => {
                #[cfg(target_arch = "x86_64")]
                {
                    // SAFETY: Backend selection checked the current CPU for AVX2.
                    $function(unsafe { $crate::xxhash::kernel::Avx2::new_unchecked() }, $($argument),*)
                }
                #[cfg(not(target_arch = "x86_64"))]
                unreachable!("AVX2 cannot be selected on this target")
            }
        }
    }};
}

pub(crate) use dispatch;
