use crate::{bitmask::BitMask, control::ControlByte};

pub trait GroupOps {
    type View: Copy;
    fn load(ptr: *const ControlByte) -> Self::View;
    fn match_tag(view: Self::View, tag: u8) -> BitMask;
    fn match_deleted(view: Self::View) -> BitMask;
    fn match_empty(view: Self::View) -> BitMask;
}

pub struct Group<G: GroupOps> {
    view: G::View,
}

impl<G: GroupOps> Group<G> {
    #[inline(always)]
    pub fn load(ptr: *const ControlByte) -> Self {
        // unwrap is fine here becuase if it ever breaks
        // there is something wrong with our invariants
        Group { view: G::load(ptr) }
    }

    #[inline(always)]
    pub fn match_tag(&self, tag: u8) -> BitMask {
        G::match_tag(self.view, tag)
    }

    #[inline(always)]
    pub fn match_deleted(&self) -> BitMask {
        G::match_deleted(self.view)
    }

    #[inline(always)]
    pub fn match_empty(&self) -> BitMask {
        G::match_empty(self.view)
    }
}
