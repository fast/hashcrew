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

pub fn next_random(state: &mut u64) -> u64 {
    *state ^= *state >> 12;
    *state ^= *state << 25;
    *state ^= *state >> 27;
    state.wrapping_mul(0x2545_f491_4f6c_dd1d)
}

pub fn random_input(state: &mut u64, len: usize) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(len);
    while bytes.len() < len {
        bytes.extend_from_slice(&next_random(state).to_le_bytes());
    }
    bytes.truncate(len);
    bytes
}
