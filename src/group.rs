use crate::{bitmask::BitMask, control::ControlByte};

pub trait GroupOps {
    fn match_tag(bytes: &[ControlByte; 16], tag: u8) -> BitMask;
    fn match_deleted(bytes: &[ControlByte; 16]) -> BitMask;
    fn match_empty(bytes: &[ControlByte; 16]) -> BitMask;
}

pub struct Group<'a> {
    bytes: &'a [ControlByte; 16],
}

impl<'a> Group<'a> {
    pub fn new(bytes: &'a [ControlByte]) -> Self {
        // unwrap is fine here becuase if it ever breaks
        // there is something wrong with our invariants
        Group {
            bytes: bytes.try_into().unwrap(),
        }
    }

    pub fn match_tag<G: GroupOps>(&self, tag: u8) -> BitMask {
        G::match_tag(self.bytes, tag)
    }

    pub fn match_deleted<G: GroupOps>(&self) -> BitMask {
        G::match_deleted(self.bytes)
    }

    pub fn match_empty<G: GroupOps>(&self) -> BitMask {
        G::match_empty(self.bytes)
    }
}
