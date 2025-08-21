use std::{
    hash::{BuildHasher, Hash, Hasher, RandomState},
    mem::{self, MaybeUninit},
};

use crate::control::{ctrl_deleted, ctrl_empty, ctrl_h2, is_deleted, is_empty, is_full};

const DEFAULT_CAPACITY: usize = 1024;

enum InsertProbe {
    Vacant(usize),
    Found(usize),
}

pub struct Miso<K, V> {
    items: Vec<MaybeUninit<(K, V)>>,
    control_bytes: Vec<u8>,
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
        let capacity = capacity.max(16).next_power_of_two();
        let mut items = Vec::with_capacity(capacity);
        items.resize_with(capacity, || MaybeUninit::uninit());
        Miso {
            items,
            hash_builder: RandomState::new(),
            control_bytes: vec![ctrl_empty(); capacity],
            capacity,
            size: 0,
            tombstones: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        self.maybe_grow();
        let (h1, h2) = self.get_h1_h2_from_key(&key);

        match self.probe_for_insert(h1, h2, &key) {
            Some(InsertProbe::Found(index)) => unsafe {
                let (_, v) = self.items[index].assume_init_mut();
                let old = mem::replace(v, value);
                Some(old)
            },
            Some(InsertProbe::Vacant(index)) => {
                if is_deleted(self.control_bytes[index]) {
                    self.tombstones -= 1
                }
                self.items[index].write((key, value));
                self.control_bytes[index] = h2;
                self.size += 1;
                None
            }
            None => {
                self.grow();
                self.insert(key, value);
                None
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let (h1, h2) = self.get_h1_h2_from_key(&key);
        match self.probe_for_lookup(h1, h2, &key) {
            Some(index) => unsafe {
                let (_, v) = self.items[index].assume_init_ref();
                Some(v)
            },
            None => None,
        }
    }

    fn probe_for_insert(&self, h1: u64, h2: u8, key: &K) -> Option<InsertProbe> {
        return self.probe_for_insert_scalar(h1, h2, key);
    }

    fn probe_for_insert_scalar(&self, h1: u64, h2: u8, key: &K) -> Option<InsertProbe> {
        let mut index = self.get_index_from_h1(h1);
        let mut first_tombstone = None;
        let start = index;

        loop {
            let ctrl = self.control_bytes[index];

            if is_full(ctrl) {
                if ctrl_h2(ctrl) == h2 {
                    unsafe {
                        let (k, _) = self.items[index].assume_init_ref();
                        if *k == *key {
                            return Some(InsertProbe::Found(index));
                        }
                    }
                }
            }

            if is_empty(ctrl) {
                if first_tombstone.is_some() {
                    return first_tombstone;
                }
                return Some(InsertProbe::Vacant(index));
            }

            if is_deleted(ctrl) && first_tombstone.is_none() {
                first_tombstone = Some(InsertProbe::Vacant(index))
            }

            index = (index + 1) & (self.capacity - 1);

            if index == start {
                return None;
            }
        }
    }

    #[inline]
    fn get_index_from_h1(&self, h1: u64) -> usize {
        (h1 as usize) & (self.capacity() - 1)
    }

    fn get_h1_h2_from_key(&self, key: &K) -> (u64, u8) {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let h1 = hash >> 7;
        let h2 = (hash >> 57) as u8 & 0x7F;

        (h1, h2)
    }

    fn probe_for_lookup(&self, h1: u64, h2: u8, key: &K) -> Option<usize> {
        debug_assert!(self.capacity.is_power_of_two());
        let mut index = self.get_index_from_h1(h1);
        let start = index;
        loop {
            let ctrl = self.control_bytes[index];

            if is_full(ctrl) {
                let ctrl_h2 = ctrl_h2(ctrl);
                if ctrl_h2 == h2 {
                    unsafe {
                        let (k, _) = &self.items[index].assume_init_ref();

                        if *k == *key {
                            return Some(index);
                        }
                    }
                }
            }
            if is_empty(ctrl) {
                return None;
            }
            index = (index + 1) & (self.capacity - 1);
            if index == start {
                return None;
            }
        }
    }

    pub fn delete(&mut self, key: &K) -> Option<V> {
        let (h1, h2) = self.get_h1_h2_from_key(&key);
        match self.probe_for_lookup(h1, h2, &key) {
            Some(index) => unsafe {
                let (_, v) = self.items[index].assume_init_read();
                self.size -= 1;
                self.tombstones += 1;
                self.control_bytes[index] = ctrl_deleted();
                return Some(v);
            },
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

        for i in 0..self.capacity {
            if is_full(self.control_bytes[i]) {
                unsafe {
                    let (k, v) = self.items[i].assume_init_read();
                    self.control_bytes[i] = ctrl_deleted();
                    new_map.insert(k, v);
                }
            }
        }

        *self = new_map
    }
}

impl<K, V> Drop for Miso<K, V> {
    fn drop(&mut self) {
        for i in 0..self.capacity {
            if is_full(self.control_bytes[i]) {
                unsafe {
                    std::ptr::drop_in_place(self.items[i].as_mut_ptr());
                }
            }
        }
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

        // Insert just past the grow threshold: floor(7/8*cap) + 1
        let need = (old_cap * 7) / 8 + 1;
        for i in 0..need {
            map.insert(format!("k{i}"), format!("v{i}"));
        }

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
