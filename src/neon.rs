use crate::{bitmask::BitMask, control::ControlByte, group::GroupOps};
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

#[cfg(all(target_arch = "arm", target_feature = "neon"))]
use std::arch::arm::*;
pub struct NeonGroup;

impl GroupOps for NeonGroup {
    fn match_tag(bytes: &[crate::control::ControlByte; 16], tag: u8) -> crate::bitmask::BitMask {
        unsafe {
            let ctrl_vec = vld1q_u8(bytes.as_ptr() as *const u8);
            let tag_vec = vdupq_n_u8(tag);
            let cmp_vec = vceqq_u8(ctrl_vec, tag_vec);
            // extract low and high parts for the reduction step
            let lo = vget_low_u8(cmp_vec);
            let hi = vget_high_u8(cmp_vec);
            // reduce to two u64s
            let weights = vcreate_u8(0x8040201008040201u64);

            let l0 = vand_u8(lo, weights);
            let h0 = vand_u8(hi, weights);

            // reduce the low parts
            let l1 = vpaddl_u8(l0);
            let l2 = vpaddl_u16(l1);
            let l3 = vpaddl_u32(l2);
            let l4 = vget_lane_u64(l3, 0) as u8;

            // reduce the high parts
            let h1 = vpaddl_u8(h0);
            let h2 = vpaddl_u16(h1);
            let h3 = vpaddl_u32(h2);
            let h4 = vget_lane_u64(h3, 0) as u8;

            let mask = l4 as u16 | ((h4 as u16) << 8);
            BitMask::from_u16(mask)
        }
    }

    fn match_empty(bytes: &[crate::control::ControlByte; 16]) -> crate::bitmask::BitMask {
        unsafe {
            let ctrl_vec = vld1q_u8(bytes.as_ptr() as *const u8);
            let tag_vec = vdupq_n_u8(ControlByte::EMPTY);
            let cmp_vec = vceqq_u8(ctrl_vec, tag_vec);
            // extract low and high parts for the reduction step
            let lo = vget_low_u8(cmp_vec);
            let hi = vget_high_u8(cmp_vec);
            // reduce to two u64s
            let weights = vcreate_u8(0x8040201008040201u64);

            let l0 = vand_u8(lo, weights);
            let h0 = vand_u8(hi, weights);

            // reduce the low parts
            let l1 = vpaddl_u8(l0);
            let l2 = vpaddl_u16(l1);
            let l3 = vpaddl_u32(l2);
            let l4 = vget_lane_u64(l3, 0) as u8;

            // reduce the high parts
            let h1 = vpaddl_u8(h0);
            let h2 = vpaddl_u16(h1);
            let h3 = vpaddl_u32(h2);
            let h4 = vget_lane_u64(h3, 0) as u8;

            let mask = l4 as u16 | ((h4 as u16) << 8);
            BitMask::from_u16(mask)
        }
    }

    fn match_deleted(bytes: &[crate::control::ControlByte; 16]) -> crate::bitmask::BitMask {
        unsafe {
            let ctrl_vec = vld1q_u8(bytes.as_ptr() as *const u8);
            let tag_vec = vdupq_n_u8(ControlByte::DELETED);
            let cmp_vec = vceqq_u8(ctrl_vec, tag_vec);
            // extract low and high parts for the reduction step
            let lo = vget_low_u8(cmp_vec);
            let hi = vget_high_u8(cmp_vec);
            // reduce to two u64s
            let weights = vcreate_u8(0x8040201008040201u64);

            let l0 = vand_u8(lo, weights);
            let h0 = vand_u8(hi, weights);

            // reduce the low parts
            let l1 = vpaddl_u8(l0);
            let l2 = vpaddl_u16(l1);
            let l3 = vpaddl_u32(l2);
            let l4 = vget_lane_u64(l3, 0) as u8;

            // reduce the high parts
            let h1 = vpaddl_u8(h0);
            let h2 = vpaddl_u16(h1);
            let h3 = vpaddl_u32(h2);
            let h4 = vget_lane_u64(h3, 0) as u8;

            let mask = l4 as u16 | ((h4 as u16) << 8);
            BitMask::from_u16(mask)
        }
    }
}
