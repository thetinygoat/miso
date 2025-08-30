#[repr(transparent)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlByte(u8);

#[derive(Debug)]
pub struct ControlBytes {
    bytes: Box<[ControlByte]>,
    capacity: usize,
}

impl ControlByte {
    pub const EMPTY: u8 = 0x80;
    pub const DELETED: u8 = 0xFE;
    pub const SENTINEL: u8 = 0xFF;

    pub const EMPTY_B: ControlByte = ControlByte(Self::EMPTY);
    pub const DELETED_B: ControlByte = ControlByte(Self::DELETED);
    pub const SENTINEL_B: ControlByte = ControlByte(Self::SENTINEL);

    #[inline(always)]
    pub fn from(byte: u8) -> Self {
        ControlByte(byte)
    }
    // the MSB is not set for full control bytes
    #[inline(always)]
    pub fn is_full(self) -> bool {
        self.0 & 0x80 == 0
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.0 == Self::EMPTY
    }

    #[inline(always)]
    pub fn is_deleted(&self) -> bool {
        self.0 == Self::DELETED
    }

    #[inline(always)]
    pub fn tag(&self) -> u8 {
        return self.0 & 0x7F;
    }

    #[inline(always)]
    pub fn from_tag(tag: u8) -> Self {
        ControlByte(tag & 0x7F)
    }
}

impl ControlBytes {
    pub fn new(capacity: usize) -> Self {
        let mut bytes: Vec<ControlByte> = vec![ControlByte::EMPTY_B; capacity + 16];
        bytes[capacity] = ControlByte::SENTINEL_B;

        debug_assert!(bytes.len() == capacity + 16);
        debug_assert!(bytes[capacity] == ControlByte::SENTINEL_B);
        ControlBytes {
            bytes: bytes.into_boxed_slice(),
            capacity,
        }
    }

    #[inline(always)]
    pub fn at(&self, idx: usize) -> &ControlByte {
        &self.bytes[idx]
    }

    #[inline(always)]
    fn clone_idx(&self, idx: usize) -> Option<usize> {
        if idx >= 15 {
            return None;
        }

        let clone_idx = self.capacity + idx + 1;

        Some(clone_idx)
    }

    #[inline(always)]
    pub fn set_full(&mut self, idx: usize, tag: u8) {
        self.bytes[idx] = ControlByte::from_tag(tag);
        if let Some(clone_idx) = self.clone_idx(idx) {
            self.bytes[clone_idx] = self.bytes[idx];
        }
    }

    #[inline(always)]
    pub fn set_deleted(&mut self, idx: usize) {
        self.bytes[idx] = ControlByte::DELETED_B;
        if let Some(clone_idx) = self.clone_idx(idx) {
            self.bytes[clone_idx] = self.bytes[idx];
        }
    }

    #[inline(always)]
    pub fn set_empty(&mut self, idx: usize) {
        self.bytes[idx] = ControlByte::EMPTY_B;
        if let Some(clone_idx) = self.clone_idx(idx) {
            self.bytes[clone_idx] = self.bytes[idx];
        }
    }

    #[inline(always)]
    pub fn ptr_at(&self, idx: usize) -> *const ControlByte {
        unsafe { self.bytes.as_ptr().add(idx) }
    }

    #[inline(always)]
    pub fn table_idx(&self, start_idx: usize, bit_position: usize) -> usize {
        let raw = start_idx + bit_position;
        let wrap = (raw >= self.capacity) as usize;

        return (raw - wrap) & (self.capacity - 1);
    }
}
