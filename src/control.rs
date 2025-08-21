const EMPTY: u8 = 0x80;
const DELETED: u8 = 0xFE;
const SENTINEL: u8 = 0xFF;

#[inline]
pub fn is_empty(ctrl: u8) -> bool {
    ctrl == EMPTY
}

#[inline]
pub fn is_deleted(ctrl: u8) -> bool {
    ctrl == DELETED
}

#[inline]
pub fn is_full(ctrl: u8) -> bool {
    ctrl & 0x80 == 0
}

#[inline]
pub fn ctrl_h2(ctrl: u8) -> u8 {
    ctrl & 0x7F
}

#[inline]
pub fn ctrl_empty() -> u8 {
    EMPTY
}

#[inline]
pub fn ctrl_deleted() -> u8 {
    DELETED
}
