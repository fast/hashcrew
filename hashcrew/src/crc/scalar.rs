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

pub(super) const fn table(polynomial: u32) -> [[u32; 256]; 8] {
    let mut table = [[0; 256]; 8];
    let mut byte = 0;
    while byte < 256 {
        let mut value = byte as u32;
        let mut bit = 0;
        while bit < 8 {
            value = (value >> 1) ^ (polynomial & 0_u32.wrapping_sub(value & 1));
            bit += 1;
        }
        table[0][byte] = value;
        byte += 1;
    }
    let mut lane = 1;
    while lane < 8 {
        let mut byte = 0;
        while byte < 256 {
            let value = table[lane - 1][byte];
            table[lane][byte] = (value >> 8) ^ table[0][(value & 0xff) as usize];
            byte += 1;
        }
        lane += 1;
    }
    table
}

#[inline]
pub(super) fn update(mut state: u32, mut input: &[u8], table: &[[u32; 256]; 8]) -> u32 {
    while input.len() >= 8 {
        let word = u32::from_le_bytes([input[0], input[1], input[2], input[3]]) ^ state;
        state = table[7][(word & 0xff) as usize]
            ^ table[6][((word >> 8) & 0xff) as usize]
            ^ table[5][((word >> 16) & 0xff) as usize]
            ^ table[4][(word >> 24) as usize]
            ^ table[3][input[4] as usize]
            ^ table[2][input[5] as usize]
            ^ table[1][input[6] as usize]
            ^ table[0][input[7] as usize];
        input = &input[8..];
    }
    for &byte in input {
        state = (state >> 8) ^ table[0][((state as u8) ^ byte) as usize];
    }
    state
}

pub(super) fn combine(mut left: u32, right: u32, mut right_len: u64, polynomial: u32) -> u32 {
    if right_len == 0 {
        return left;
    }

    // Columns of the linear transformation that advances a register by one
    // zero byte. Squaring doubles the number of bytes represented by the matrix.
    let mut matrix = core::array::from_fn(|bit| {
        let mut value = 1_u32 << bit;
        for _ in 0..8 {
            value = (value >> 1) ^ (polynomial & 0_u32.wrapping_sub(value & 1));
        }
        value
    });
    loop {
        if right_len & 1 != 0 {
            left = apply(&matrix, left);
        }
        right_len >>= 1;
        if right_len == 0 {
            break;
        }
        matrix = core::array::from_fn(|bit| apply(&matrix, matrix[bit]));
    }
    // Both supported variants use equal init and xorout values, so the affine
    // correction cancels when combining finalized checksums.
    left ^ right
}

fn apply(matrix: &[u32; 32], mut value: u32) -> u32 {
    let mut result = 0;
    while value != 0 {
        let bit = value.trailing_zeros();
        result ^= matrix[bit as usize];
        value &= value - 1;
    }
    result
}
