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

#[inline(always)]
pub(crate) fn read_u32(input: &[u8], offset: usize) -> u32 {
    let bytes: [u8; 4] = input[offset..offset + 4]
        .try_into()
        .expect("validated hash input range");
    u32::from_le_bytes(bytes)
}

#[inline(always)]
pub(crate) fn read_u64(input: &[u8], offset: usize) -> u64 {
    let bytes: [u8; 8] = input[offset..offset + 8]
        .try_into()
        .expect("validated hash input range");
    u64::from_le_bytes(bytes)
}

#[inline(always)]
pub(crate) fn mul128_fold64(lhs: u64, rhs: u64) -> u64 {
    let product = u128::from(lhs) * u128::from(rhs);
    product as u64 ^ (product >> 64) as u64
}
