#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::{
    uint8x16_t, vaddv_u8, vandq_u8, vceqq_u8, vdupq_n_u8, vget_high_u8, vget_low_u8, vld1q_u8,
};
use std::hash::{BuildHasher, Hash, Hasher, RandomState};

const DEFAULT_CAPACITY: usize = 1024;
const EMPTY: u8 = 0x80;
const DELETED: u8 = 0xFE;

const LOOKUP_TABLE: [u8; 16] = [
    0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x01, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80,
];
pub struct Miso<K, V> {
    items: Vec<Option<(K, V)>>,
    metadata: Vec<u8>,
    hash_builder: RandomState,
    capacity: usize,
    size: usize,
    tombstones: usize,
}

impl<K, V> Miso<K, V> {
    pub fn size(&self) -> usize {
        return self.size;
    }

    pub fn capacity(&self) -> usize {
        return self.capacity;
    }
}

impl<K, V> Miso<K, V>
where
    K: Hash + Eq,
{
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_CAPACITY)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        let mut items = Vec::with_capacity(capacity);
        items.resize_with(capacity, || None);
        Miso {
            items,
            hash_builder: RandomState::new(),
            metadata: vec![EMPTY; capacity],
            capacity,
            size: 0,
            tombstones: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        self.maybe_grow();
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        match self.probe_for_insert(hash, &key) {
            Some((index, new)) => {
                let was_deleted = self.metadata[index] == DELETED;
                self.items[index] = Some((key, value));
                let hash_fingerprint = ((hash >> 57) & (0x7F)) as u8;
                self.metadata[index] = hash_fingerprint;
                if new {
                    self.size += 1;
                }

                if was_deleted {
                    self.tombstones -= 1
                }
            }
            None => {
                self.grow();
                self.insert(key, value);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        match self.probe_for_lookup(hash, &key) {
            Some(index) => self.items[index].as_ref().map(|(_, v)| v),
            None => None,
        }
    }

    fn probe_for_insert(&self, hash: u64, key: &K) -> Option<(usize, bool)> {
        #[cfg(target_arch = "aarch64")]
        {
            return self.probe_for_insert_aarch64(hash, key);
        }

        #[cfg(not(target_arch = "aarch64"))]
        {
            return self.probe_for_insert_default(hash, key);
        }
    }

    #[allow(dead_code)]
    fn probe_for_insert_default(&self, hash: u64, key: &K) -> Option<(usize, bool)> {
        let mut index = ((hash) & (self.capacity - 1) as u64) as usize;
        let original_index = index;
        let mut reusable_index = None;
        let hash_fingerprint = ((hash >> 57) & 0x7F) as u8;
        loop {
            let control = self.metadata[index];
            if control == EMPTY {
                return match reusable_index {
                    Some(idx) => Some((idx, true)),
                    None => Some((index, true)),
                };
            }

            if control == DELETED {
                if reusable_index.is_none() {
                    reusable_index = Some(index);
                }

                index = (index + 1) & (self.capacity - 1);

                if index == original_index {
                    return match reusable_index {
                        Some(idx) => Some((idx, true)),
                        None => None,
                    };
                }

                continue;
            }

            let stored_fingerprint = control & 0x7F;
            // extract the top 7 bits of the hash

            // we might have a match, check the exact key
            if stored_fingerprint == hash_fingerprint {
                // the other case cannot happen
                if let Some((k, _)) = &self.items[index] {
                    if *key == *k {
                        return Some((index, false));
                    }
                }
            }

            index = (index + 1) & (self.capacity - 1);

            if index == original_index {
                return match reusable_index {
                    Some(idx) => Some((idx, true)),
                    None => None,
                };
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    fn probe_for_insert_aarch64(&self, hash: u64, key: &K) -> Option<(usize, bool)> {
        unsafe {
            let mut index = ((hash) & (self.capacity - 1) as u64) as usize;
            let original_index = index;
            let mut reusable_index = None;
            let hash_fingerprint = vdupq_n_u8(((hash >> 57) & 0x7F) as u8);
            loop {
                // we cannot load 16 bytes into the simd register, fallback to scalar
                if index + 16 >= self.capacity {
                    return self.probe_for_insert_default(hash, key);
                }
                let metadata_vector = vld1q_u8(self.metadata.as_ptr().add(index));
                let empty_mask = self.get_mask(vceqq_u8(metadata_vector, vdupq_n_u8(EMPTY)));
                let mut delete_mask = self.get_mask(vceqq_u8(metadata_vector, vdupq_n_u8(DELETED)));
                let mut hash_mask = self.get_mask(vceqq_u8(metadata_vector, hash_fingerprint));

                // process the 16 bits of the mask
                while empty_mask != 0 || delete_mask != 0 || hash_mask != 0 {
                    if empty_mask != 0 {
                        let offset = empty_mask.trailing_zeros() as usize;
                        return match reusable_index {
                            Some(idx) => Some((idx, true)),
                            None => Some((index + offset, true)),
                        };
                    }

                    if reusable_index.is_none() && delete_mask != 0 {
                        let offset = delete_mask.trailing_zeros() as usize;
                        reusable_index = Some(index + offset);
                        delete_mask &= delete_mask - 1;
                    }

                    if hash_mask != 0 {
                        let offset = hash_mask.trailing_zeros() as usize;

                        if let Some((k, _)) = &self.items[index + offset] {
                            if *key == *k {
                                return Some((index + offset, false));
                            }
                        }

                        hash_mask &= hash_mask - 1;
                    }
                }

                index = (index + 16) & (self.capacity - 1);
                if index == original_index {
                    return match reusable_index {
                        Some(idx) => Some((idx, true)),
                        None => None,
                    };
                }
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    unsafe fn get_mask(&self, result: uint8x16_t) -> u16 {
        unsafe {
            // load the lookup table into 16 lanes
            let lookup = vld1q_u8(LOOKUP_TABLE.as_ptr());
            // AND the matches with the lookup table to set proper bits
            let masked = vandq_u8(result, lookup);
            // convert the high and low vectors to bits to a single u8 with proper bits set
            let low = vaddv_u8(vget_low_u8(masked));
            let high = vaddv_u8(vget_high_u8(masked));
            // construct the final bitmask
            low as u16 | ((high as u16) << 8)
        }
    }

    fn probe_for_lookup(&self, hash: u64, key: &K) -> Option<usize> {
        let mut index = ((hash) & (self.capacity - 1) as u64) as usize;
        let original_index = index;
        loop {
            let control = self.metadata[index];

            if control == EMPTY {
                return None;
            }

            if control == DELETED {
                index = (index + 1) & (self.capacity - 1);
                if index == original_index {
                    return None;
                }
                continue;
            }

            let stored_fingerprint = control & 0x7F;
            // extract the top 7 bits of the hash
            let hash_fingerprint = ((hash >> 57) & 0x7F) as u8;

            // we might have a match, check the exact key
            if stored_fingerprint == hash_fingerprint {
                // the other case cannot happen
                if let Some((k, _)) = &self.items[index] {
                    if *key == *k {
                        return Some(index);
                    }
                }
            }

            index = (index + 1) & (self.capacity - 1);

            if index == original_index {
                return None;
            }
        }
    }

    pub fn delete(&mut self, key: &K) -> Option<V> {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        match self.probe_for_lookup(hash, &key) {
            Some(index) => {
                let item = self.items[index].take().map(|(_, v)| v);
                self.items[index] = None;
                self.size -= 1;
                self.tombstones += 1;
                self.metadata[index] = DELETED;
                return item;
            }
            None => None,
        }
    }

    fn maybe_grow(&mut self) {
        let should_resize = (self.tombstones + self.size) * 8 >= self.capacity * 7;
        if should_resize {
            self.grow();
        }
    }

    fn grow(&mut self) {
        let mut new_map = Miso::with_capacity(2 * self.capacity);
        new_map.hash_builder = self.hash_builder.clone();

        for item in self.items.iter_mut() {
            if let Some((k, v)) = item.take() {
                new_map.insert(k, v);
            }
        }

        *self = new_map
    }
}

#[cfg(test)]
mod tests {
    use crate::miso::Miso;
    #[test]
    fn test_insert() {
        let mut map = Miso::with_capacity(2);
        let key = "key";
        let value = "value";
        map.insert(key, value);
        assert_eq!(map.get(&key), Some(&value));
        assert_eq!(map.size(), 1);
    }

    #[test]
    fn test_resize() {
        let mut map = Miso::with_capacity(2);
        let old_cap = map.capacity();
        let key1 = "key1";
        let value1 = "value1";
        let key2 = "key2";
        let value2 = "value1";
        let key3 = "key3";
        let value3 = "value1";
        map.insert(key1, value1);
        map.insert(key2, value2);
        map.insert(key3, value3);
        assert_eq!(map.get(&key1), Some(&value1));
        assert_eq!(map.get(&key2), Some(&value2));
        assert_eq!(map.get(&key3), Some(&value3));
        assert!(map.capacity() > old_cap);
    }

    #[test]
    fn test_delete() {
        let mut map = Miso::new();
        let key = "key";
        let value = "value";
        map.insert(key, value);
        map.delete(&key);
        assert_eq!(map.get(&key), None);
        assert_eq!(map.size(), 0);
    }

    #[test]
    fn test_duplicate_hash() {
        let mut map = Miso::with_capacity(4);
        let key = "key";
        let value1 = "value1";
        let value2 = "value2";
        map.insert(key, value1);
        map.insert(key, value2);
    }
}
