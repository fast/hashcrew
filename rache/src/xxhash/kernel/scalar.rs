// Copyright 2026 rache contributors
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

use super::Xxh3Kernel;
use crate::util::read_u64;

const PRIME32_1: u64 = 0x9e37_79b1;

#[derive(Clone, Copy)]
pub(crate) struct Scalar;

impl Xxh3Kernel for Scalar {
    #[inline]
    fn accumulate(self, acc: &mut [u64; 8], stripe: &[u8; 64], secret: &[u8; 64]) {
        for lane in 0..8 {
            let data = read_u64(stripe, lane * 8);
            let keyed = data ^ read_u64(secret, lane * 8);
            acc[lane ^ 1] = acc[lane ^ 1].wrapping_add(data);
            acc[lane] = acc[lane].wrapping_add(
                u64::from(keyed as u32).wrapping_mul(u64::from((keyed >> 32) as u32)),
            );
        }
    }

    #[inline]
    fn scramble(self, acc: &mut [u64; 8], secret: &[u8; 64]) {
        for (lane, value) in acc.iter_mut().enumerate() {
            *value ^= *value >> 47;
            *value ^= read_u64(secret, lane * 8);
            *value = value.wrapping_mul(PRIME32_1);
        }
    }
}
