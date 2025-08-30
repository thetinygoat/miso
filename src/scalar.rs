use crate::{bitmask::BitMask, control::ControlByte, group::GroupOps};

pub struct ScalarOps;

impl GroupOps for ScalarOps {
    type View = [u8; 16];
    #[inline(always)]
    fn load(ptr: *const ControlByte) -> Self::View {
        let p = ptr as *const u8;
        let mut buf = [0u8; 16];
        unsafe {
            std::ptr::copy_nonoverlapping(p, buf.as_mut_ptr(), 16);
            buf
        }
    }
    #[inline(always)]
    fn match_tag(view: Self::View, tag: u8) -> BitMask {
        let mut mask = BitMask::ZERO;

        for i in 0..16 {
            let byte = ControlByte::from(view[i]);
            if byte.is_full() {
                if byte.tag() == tag {
                    mask.set(i);
                }
            }
        }

        mask
    }

    #[inline(always)]
    fn match_deleted(view: Self::View) -> BitMask {
        let mut mask = BitMask::ZERO;

        for i in 0..16 {
            if ControlByte::from(view[i]).is_deleted() {
                mask.set(i);
            }
        }

        mask
    }

    #[inline(always)]
    fn match_empty(view: Self::View) -> BitMask {
        let mut mask = BitMask::ZERO;

        for i in 0..16 {
            if ControlByte::from(view[i]).is_empty() {
                mask.set(i);
            }
        }

        mask
    }
}
