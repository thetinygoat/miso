#[derive(Debug, Clone)]
pub struct BitMask(u16);

impl BitMask {
    pub const ZERO: BitMask = BitMask(0);

    #[inline(always)]
    pub fn any(&self) -> bool {
        self.0 != 0
    }

    #[inline(always)]
    pub fn lsb_idx(&self) -> usize {
        self.0.trailing_zeros() as usize
    }

    #[inline(always)]
    pub fn pop_lsb(&mut self) -> Option<usize> {
        if self.0 == 0 {
            return None;
        }
        let idx = self.lsb_idx();
        self.0 &= self.0 - 1;
        Some(idx)
    }

    #[inline(always)]
    pub fn set(&mut self, i: usize) {
        self.0 |= 1 << i;
    }

    #[inline(always)]
    pub fn from_u16(mask: u16) -> BitMask {
        BitMask(mask)
    }
}
