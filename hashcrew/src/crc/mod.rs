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

//! CRC checksums for existing formats and protocols.
//!
//! | Variant         | One-shot function  | Incremental state | Also known as              |
//! | --------------- | ------------------ | ----------------- | -------------------------- |
//! | CRC-32/ISO-HDLC | [`crc32_iso_hdlc`] | [`Crc32IsoHdlc`]  | IEEE CRC32, Ethernet CRC32 |
//! | CRC-32/ISCSI    | [`crc32_iscsi`]    | [`Crc32Iscsi`]    | CRC32C, Castagnoli         |
//!
//! These variants have different polynomials and are not interchangeable. Both
//! reflect input and output, initialize the register to `0xffffffff`, apply a
//! final XOR of `0xffffffff`, and return zero for an empty input. Results are
//! independent of input alignment, update boundaries, and platform byte order.
//! Use the byte order required by the consuming format when storing a checksum.
//! CRCs detect accidental corruption; they do not authenticate data.
//!
//! States retain only a running checksum. Call `digest` repeatedly or continue
//! updating afterward. `from_digest` resumes from a finalized checksum of the
//! same variant, rather than accepting a raw register value or custom seed.
//! With the `std` feature, states also implement [`std::io::Write`].
//!
//! ```
//! use hashcrew::crc::Crc32Iscsi;
//! use hashcrew::crc::crc32_iscsi;
//! use hashcrew::crc::crc32_iscsi_combine;
//!
//! let mut state = Crc32Iscsi::new();
//! state.update(b"1234");
//! let prefix = state.digest();
//! state.update(b"56789");
//! assert_eq!(state.digest(), 0xe306_9283);
//!
//! let mut resumed = Crc32Iscsi::from_digest(prefix);
//! resumed.update(b"56789");
//! assert_eq!(resumed.digest(), state.digest());
//!
//! let combined = crc32_iscsi_combine(prefix, crc32_iscsi(b"56789"), 5);
//! assert_eq!(combined, state.digest());
//! ```

mod scalar;

// Reflected forms of the catalogue polynomials 0x04c11db7 and 0x1edc6f41.
const ISO_HDLC_POLYNOMIAL: u32 = 0xedb8_8320;
const ISCSI_POLYNOMIAL: u32 = 0x82f6_3b78;
static ISO_HDLC_TABLE: [[u32; 256]; 8] = scalar::table(ISO_HDLC_POLYNOMIAL);
static ISCSI_TABLE: [[u32; 256]; 8] = scalar::table(ISCSI_POLYNOMIAL);

/// Computes CRC-32/ISO-HDLC (IEEE CRC32) for `input`.
/// Returns `0xcbf43926` for `b"123456789"` and zero for empty input.
#[doc(alias = "crc32")]
#[must_use]
#[inline]
pub fn crc32_iso_hdlc(input: &[u8]) -> u32 {
    let mut state = Crc32IsoHdlc::new();
    state.update(input);
    state.digest()
}

/// Computes CRC-32/ISCSI (CRC32C, Castagnoli) for `input`.
/// Returns `0xe3069283` for `b"123456789"` and zero for empty input.
#[doc(alias = "crc32c")]
#[must_use]
#[inline]
pub fn crc32_iscsi(input: &[u8]) -> u32 {
    let mut state = Crc32Iscsi::new();
    state.update(input);
    state.digest()
}

/// Incremental CRC-32/ISO-HDLC (IEEE CRC32) checksum.
///
/// [`Self::digest`] preserves the state and includes the standard final XOR.
#[doc(alias = "Crc32")]
#[derive(Clone, Debug)]
pub struct Crc32IsoHdlc {
    state: u32,
}

impl Crc32IsoHdlc {
    /// Creates an empty CRC-32/ISO-HDLC state.
    #[must_use]
    pub const fn new() -> Self {
        Self { state: u32::MAX }
    }

    /// Resumes CRC-32/ISO-HDLC from the finalized checksum of a byte prefix.
    /// Further updates append bytes to that prefix. Passing zero is equivalent
    /// to [`Self::new`]. No prefix length is needed.
    #[must_use]
    pub const fn from_digest(digest: u32) -> Self {
        Self { state: !digest }
    }

    /// Appends raw bytes to the message. An empty slice leaves the state unchanged.
    #[inline]
    pub fn update(&mut self, input: &[u8]) {
        self.state = scalar::update(self.state, input, &ISO_HDLC_TABLE);
    }

    /// Returns the finalized checksum of all bytes so far, allowing further updates.
    #[must_use]
    pub const fn digest(&self) -> u32 {
        !self.state
    }

    /// Resets the state to an empty message.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for Crc32IsoHdlc {
    fn default() -> Self {
        Self::new()
    }
}

/// Incremental CRC-32/ISCSI (CRC32C, Castagnoli) checksum.
///
/// [`Self::digest`] preserves the state and includes the standard final XOR.
#[doc(alias = "Crc32c")]
#[derive(Clone, Debug)]
pub struct Crc32Iscsi {
    state: u32,
}

impl Crc32Iscsi {
    /// Creates an empty CRC-32/ISCSI state.
    #[must_use]
    pub const fn new() -> Self {
        Self { state: u32::MAX }
    }

    /// Resumes CRC-32/ISCSI from the finalized checksum of a byte prefix.
    /// Further updates append bytes to that prefix. Passing zero is equivalent
    /// to [`Self::new`]. No prefix length is needed.
    #[must_use]
    pub const fn from_digest(digest: u32) -> Self {
        Self { state: !digest }
    }

    /// Appends raw bytes to the message. An empty slice leaves the state unchanged.
    #[inline]
    pub fn update(&mut self, input: &[u8]) {
        self.state = scalar::update(self.state, input, &ISCSI_TABLE);
    }

    /// Returns the finalized checksum of all bytes so far, allowing further updates.
    #[must_use]
    pub const fn digest(&self) -> u32 {
        !self.state
    }

    /// Resets the state to an empty message.
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

impl Default for Crc32Iscsi {
    fn default() -> Self {
        Self::new()
    }
}

/// Computes CRC-32/ISO-HDLC of `A || B` from the finalized checksums of `A`
/// and `B`, and the length of `B` in bytes, without reading either input.
///
/// Both checksums must use CRC-32/ISO-HDLC and `right_len` must be the exact
/// length of `B`. These requirements cannot be checked from the checksum values.
/// A zero `right_len` returns `left`. Runtime is logarithmic in `right_len`.
#[must_use]
pub fn crc32_iso_hdlc_combine(left: u32, right: u32, right_len: u64) -> u32 {
    scalar::combine(left, right, right_len, ISO_HDLC_POLYNOMIAL)
}

/// Computes CRC-32/ISCSI of `A || B` from the finalized checksums of `A`
/// and `B`, and the length of `B` in bytes, without reading either input.
///
/// Both checksums must use CRC-32/ISCSI and `right_len` must be the exact
/// length of `B`. These requirements cannot be checked from the checksum values.
/// A zero `right_len` returns `left`. Runtime is logarithmic in `right_len`.
#[must_use]
pub fn crc32_iscsi_combine(left: u32, right: u32, right_len: u64) -> u32 {
    scalar::combine(left, right, right_len, ISCSI_POLYNOMIAL)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_check_values_and_resumed_streams() {
        assert_eq!(crc32_iso_hdlc(b"123456789"), 0xcbf4_3926);
        assert_eq!(crc32_iscsi(b"123456789"), 0xe306_9283);
        assert_eq!(crc32_iso_hdlc(b""), 0);
        assert_eq!(crc32_iscsi(b""), 0);

        let mut ieee = Crc32IsoHdlc::from_digest(crc32_iso_hdlc(b"1234"));
        let mut castagnoli = Crc32Iscsi::from_digest(crc32_iscsi(b"1234"));
        ieee.update(b"56789");
        castagnoli.update(b"56789");
        assert_eq!(ieee.digest(), 0xcbf4_3926);
        assert_eq!(castagnoli.digest(), 0xe306_9283);
        assert_eq!(
            crc32_iso_hdlc_combine(crc32_iso_hdlc(b"1234"), crc32_iso_hdlc(b"56789"), 5),
            ieee.digest()
        );
        assert_eq!(
            crc32_iscsi_combine(crc32_iscsi(b"1234"), crc32_iscsi(b"56789"), 5),
            castagnoli.digest()
        );
    }
}
