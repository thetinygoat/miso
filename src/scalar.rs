use crate::{bitmask::BitMask, control::ControlByte, group::GroupOps};

pub struct ScalarGroup;

impl GroupOps for ScalarGroup {
    fn match_tag(bytes: &[ControlByte; 16], tag: u8) -> BitMask {
        let mut mask = BitMask::ZERO;

        for i in 0..16 {
            let byte = &bytes[i];
            if byte.is_full() {
                if byte.tag() == tag {
                    mask.set(i);
                }
            }
        }

        mask
    }

    fn match_deleted(bytes: &[ControlByte; 16]) -> BitMask {
        let mut mask = BitMask::ZERO;

        for i in 0..16 {
            if bytes[i].is_deleted() {
                mask.set(i);
            }
        }

        mask
    }

    fn match_empty(bytes: &[ControlByte; 16]) -> BitMask {
        let mut mask = BitMask::ZERO;

        for i in 0..16 {
            if bytes[i].is_empty() {
                mask.set(i);
            }
        }

        mask
    }
}
