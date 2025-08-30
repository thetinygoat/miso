#[derive(Debug, Clone)]
pub struct BitMask(u16);

impl BitMask {
    pub const ZERO: BitMask = BitMask(0);

    #[inline]
    pub fn any(&self) -> bool {
        self.0 != 0
    }

    #[inline]
    pub fn lsb_idx(&self) -> usize {
        self.0.trailing_zeros() as usize
    }

    #[inline]
    pub fn set(&mut self, i: usize) {
        self.0 |= 1 << i;
    }

    #[inline]
    pub fn from_u16(mask: u16) -> BitMask {
        BitMask(mask)
    }
}

pub struct BitMaskIter {
    mask: u16,
}

impl Iterator for BitMaskIter {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.mask == 0 {
            return None;
        }

        let idx = self.mask.trailing_zeros() as usize;
        self.mask &= self.mask - 1;
        Some(idx)
    }
}

impl IntoIterator for BitMask {
    type Item = usize;
    type IntoIter = BitMaskIter;
    fn into_iter(self) -> Self::IntoIter {
        BitMaskIter { mask: self.0 }
    }
}
