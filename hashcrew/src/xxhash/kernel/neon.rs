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

use core::arch::aarch64::*;

use super::Xxh3Kernel;

const PRIME32_1: u32 = 0x9e37_79b1;

#[derive(Clone, Copy)]
pub(crate) struct Neon(());

impl Neon {
    pub(crate) unsafe fn new_unchecked() -> Self {
        Self(())
    }
}

impl Xxh3Kernel for Neon {
    fn accumulate(self, acc: &mut [u64; 8], stripe: &[u8; 64], secret: &[u8; 64]) {
        // SAFETY: This type is only constructed after NEON feature detection.
        unsafe { accumulate_neon(acc, stripe, secret) }
    }

    fn scramble(self, acc: &mut [u64; 8], secret: &[u8; 64]) {
        // SAFETY: This type is only constructed after NEON feature detection.
        unsafe { scramble_neon(acc, secret) }
    }
}

#[target_feature(enable = "neon")]
unsafe fn accumulate_neon(acc: &mut [u64; 8], stripe: &[u8; 64], secret: &[u8; 64]) {
    for index in 0..2 {
        let lane = index * 4;
        // SAFETY: All pointers address four lanes within fixed-size arrays, and the
        // caller guarantees NEON support. NEON loads accept unaligned addresses.
        unsafe {
            let acc_ptr = acc.as_mut_ptr().add(lane);
            let data_ptr = stripe.as_ptr().cast::<u64>().add(lane);
            let secret_ptr = secret.as_ptr().cast::<u64>().add(lane);
            let data0 = vld1q_u64(data_ptr);
            let data1 = vld1q_u64(data_ptr.add(2));
            let keyed0 = veorq_u64(data0, vld1q_u64(secret_ptr));
            let keyed1 = veorq_u64(data1, vld1q_u64(secret_ptr.add(2)));
            let parts0 = vreinterpretq_u32_u64(keyed0);
            let parts1 = vreinterpretq_u32_u64(keyed1);
            let low = vuzp1q_u32(parts0, parts1);
            let high = vuzp2q_u32(parts0, parts1);
            let sum0 = vmlal_u32(
                vextq_u64::<1>(data0, data0),
                vget_low_u32(low),
                vget_low_u32(high),
            );
            let sum1 = vmlal_high_u32(vextq_u64::<1>(data1, data1), low, high);
            scheduling_barrier(sum0);
            scheduling_barrier(sum1);
            vst1q_u64(acc_ptr, vaddq_u64(vld1q_u64(acc_ptr), sum0));
            vst1q_u64(acc_ptr.add(2), vaddq_u64(vld1q_u64(acc_ptr.add(2)), sum1));
        }
    }
}

#[target_feature(enable = "neon")]
unsafe fn scramble_neon(acc: &mut [u64; 8], secret: &[u8; 64]) {
    for index in 0..4 {
        let lane = index * 2;
        // SAFETY: All pointers address two lanes within fixed-size arrays, and the
        // caller guarantees NEON support. NEON loads accept unaligned addresses.
        unsafe {
            let acc_ptr = acc.as_mut_ptr().add(lane);
            let mut value = vld1q_u64(acc_ptr);
            value = veorq_u64(value, vshrq_n_u64::<47>(value));
            value = veorq_u64(value, vld1q_u64(secret.as_ptr().cast::<u64>().add(lane)));
            vst1q_u64(acc_ptr, mul_u64_by_u32(value, PRIME32_1));
        }
    }
}

#[target_feature(enable = "neon")]
#[inline]
unsafe fn scheduling_barrier(value: uint64x2_t) {
    // SAFETY: The empty assembly statement only creates a data dependency for
    // instruction scheduling and does not read or modify memory.
    unsafe {
        core::arch::asm!(
            "/* {value:v} */",
            value = in(vreg) value,
            options(nomem, nostack),
        );
    }
}

#[inline(always)]
unsafe fn mul_u64_by_u32(input: uint64x2_t, factor: u32) -> uint64x2_t {
    // SAFETY: These intrinsics operate only on vector values and require NEON,
    // which is guaranteed by the caller of the surrounding target-feature function.
    unsafe {
        let input_parts = vreinterpretq_u32_u64(input);
        let high_factor = vreinterpretq_u32_u64(vmovq_n_u64(u64::from(factor) << 32));
        let high_product = vreinterpretq_u64_u32(vmulq_u32(input_parts, high_factor));
        vmlal_u32(high_product, vmovn_u64(input), vmov_n_u32(factor))
    }
}
