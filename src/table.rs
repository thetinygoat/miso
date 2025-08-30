use std::{
    hash::{BuildHasher, Hash, Hasher, RandomState},
    mem::{self, MaybeUninit},
};

#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "arm", target_feature = "neon")
)))]
use crate::scalar::ScalarOps;

#[cfg(any(
    target_arch = "aarch64",
    all(target_arch = "arm", target_feature = "neon")
))]
use crate::neon::NeonOps;

use crate::{control::ControlBytes, group::Group};

#[cfg(any(
    target_arch = "aarch64",
    all(target_arch = "arm", target_feature = "neon")
))]
type DefaultOps = NeonOps;

#[cfg(not(any(
    target_arch = "aarch64",
    all(target_arch = "arm", target_feature = "neon")
)))]
type DefaultOps = ScalarOps;

const DEFAULT_CAPACITY: usize = 1024;

enum InsertProbe {
    Vacant(usize),
    Found(usize),
}

pub struct HashMap<K, V> {
    items: Vec<MaybeUninit<(K, V)>>,
    ctrl_bytes: ControlBytes,
    hash_builder: RandomState,
    capacity: usize,
    size: usize,
    tombstones: usize,
}

impl<K, V> HashMap<K, V> {
    #[inline]
    pub fn len(&self) -> usize {
        return self.size;
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        return self.capacity;
    }
}

impl<K, V> HashMap<K, V>
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
        let ctrl_bytes = ControlBytes::new(capacity);

        debug_assert!(items.len() == capacity);

        HashMap {
            items,
            hash_builder: RandomState::new(),
            ctrl_bytes,
            capacity,
            size: 0,
            tombstones: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        loop {
            self.maybe_grow_or_rehash();
            let (h1, h2) = self.get_h1_h2_from_key(&key);
            match self.probe_for_insert(h1, h2, &key) {
                Some(InsertProbe::Found(index)) => unsafe {
                    let (_, v) = self.items.get_unchecked_mut(index).assume_init_mut();
                    let old = mem::replace(v, value);
                    return Some(old);
                },
                Some(InsertProbe::Vacant(index)) => {
                    if self.ctrl_bytes.at(index).is_deleted() {
                        self.tombstones -= 1
                    }
                    unsafe {
                        self.items.get_unchecked_mut(index).write((key, value));
                    }
                    self.ctrl_bytes.set_full(index, h2);
                    self.size += 1;
                    return None;
                }
                None => {
                    // if we receieved None, that means there is no place for the new key to go
                    // we just need to continue the loop and the maybe_grow_or_rehash fn
                    // will take care of the rest
                    continue;
                }
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let (h1, h2) = self.get_h1_h2_from_key(&key);
        match self.probe_for_lookup(h1, h2, &key) {
            Some(index) => unsafe {
                let (_, v) = self.items.get_unchecked(index).assume_init_ref();
                Some(v)
            },
            None => None,
        }
    }

    fn probe_for_insert(&self, h1: u64, h2: u8, key: &K) -> Option<InsertProbe> {
        let mut index = self.get_index_from_h1(h1);
        let mut first_tombstone = None;
        let start = index;
        let mask = self.capacity() - 1;
        loop {
            let group = Group::<DefaultOps>::load(self.ctrl_bytes.ptr_at(index));
            let mut tag_mask = group.match_tag(h2);
            let delete_mask = group.match_deleted();
            let empty_mask = group.match_empty();

            if tag_mask.any() {
                while let Some(hit) = tag_mask.pop_lsb() {
                    let index = self.ctrl_bytes.table_idx(index, hit);
                    unsafe {
                        let (k, _) = self.items[index].assume_init_ref();
                        if k == key {
                            return Some(InsertProbe::Found(index));
                        }
                    }
                }
            }

            if first_tombstone.is_none() {
                if delete_mask.any() {
                    let index = self.ctrl_bytes.table_idx(index, delete_mask.lsb_idx());
                    first_tombstone = Some(InsertProbe::Vacant(index))
                }
            }

            if empty_mask.any() {
                if first_tombstone.is_some() {
                    return first_tombstone;
                }
                let index = self.ctrl_bytes.table_idx(index, empty_mask.lsb_idx());
                return Some(InsertProbe::Vacant(index));
            }

            index = (index + 16) & mask;

            if index == start {
                return first_tombstone;
            }
        }
    }

    #[inline]
    fn get_index_from_h1(&self, h1: u64) -> usize {
        debug_assert!(self.capacity.is_power_of_two());
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
        let mut index = self.get_index_from_h1(h1);
        let start = index;
        let mask = self.capacity() - 1;
        loop {
            let group = Group::<DefaultOps>::load(self.ctrl_bytes.ptr_at(index));
            let mut tag_mask = group.match_tag(h2);
            let empty_mask = group.match_empty();

            if tag_mask.any() {
                while let Some(hit) = tag_mask.pop_lsb() {
                    let index = self.ctrl_bytes.table_idx(index, hit);
                    unsafe {
                        let (k, _) = self.items.get_unchecked(index).assume_init_ref();
                        if k == key {
                            return Some(index);
                        }
                    }
                }
            }

            if empty_mask.any() {
                return None;
            }

            index = (index + 16) & mask;
            if index == start {
                return None;
            }
        }
    }

    pub fn delete(&mut self, key: &K) -> Option<V> {
        let (h1, h2) = self.get_h1_h2_from_key(&key);
        match self.probe_for_lookup(h1, h2, &key) {
            Some(index) => unsafe {
                let (_, v) = self.items.get_unchecked(index).assume_init_read();
                self.size -= 1;
                self.tombstones += 1;
                self.ctrl_bytes.set_deleted(index);
                return Some(v);
            },
            None => None,
        }
    }

    fn maybe_grow_or_rehash(&mut self) {
        if self.should_rehash() {
            self.rehash();
        } else if self.should_grow() {
            self.grow();
        }
    }

    #[inline]
    fn should_rehash(&self) -> bool {
        // Prefer grow over rehash when close to capacity
        let should_grow = (self.tombstones + self.size) >= self.capacity - (self.capacity >> 3);
        // Only rehash when we actually have tombstones and they are at least half of live items
        let should_rehash = self.tombstones > 0 && self.tombstones >= (self.size >> 1);

        should_rehash && !should_grow
    }

    #[inline]
    fn should_grow(&self) -> bool {
        // Grow if the next insertion would push load factor beyond 7/8
        (self.tombstones + self.size + 1) * 8 >= self.capacity * 7
    }

    fn grow(&mut self) {
        let mut new_map = HashMap::with_capacity(2 * self.capacity);

        new_map.hash_builder = self.hash_builder.clone();

        for i in 0..self.capacity {
            if self.ctrl_bytes.at(i).is_full() {
                unsafe {
                    let (k, v) = self.items.get_unchecked(i).assume_init_read();
                    // Mark as empty to avoid double-drop in our Drop impl
                    self.ctrl_bytes.set_empty(i);
                    new_map.insert(k, v);
                }
            }
        }

        *self = new_map
    }

    fn rehash(&mut self) {
        let mut new_map = HashMap::with_capacity(self.capacity());

        new_map.hash_builder = self.hash_builder.clone();

        for i in 0..self.capacity {
            if self.ctrl_bytes.at(i).is_full() {
                unsafe {
                    let (k, v) = self.items.get_unchecked(i).assume_init_read();
                    // Mark as empty to avoid double-drop in our Drop impl
                    self.ctrl_bytes.set_empty(i);
                    new_map.insert(k, v);
                }
            }
        }

        *self = new_map
    }
}

impl<K, V> Drop for HashMap<K, V> {
    fn drop(&mut self) {
        for i in 0..self.capacity {
            if self.ctrl_bytes.at(i).is_full() {
                unsafe {
                    std::ptr::drop_in_place(self.items[i].as_mut_ptr());
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::table::HashMap;
    #[test]
    fn test_insert() {
        let mut map = HashMap::with_capacity(2);
        let key = "key";
        let value = "value";
        map.insert(key, value);
        assert_eq!(map.get(&key), Some(&value));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_resize() {
        let mut map = HashMap::with_capacity(2);
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
        let mut map = HashMap::new();
        let key = "key";
        let value = "value";
        map.insert(key, value);
        map.delete(&key);
        assert_eq!(map.get(&key), None);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_overwrite() {
        let mut map = HashMap::with_capacity(4);
        let key = "key";
        let value1 = "value1";
        let value2 = "value2";
        assert_eq!(map.insert(key, value1), None);
        assert_eq!(map.insert(key, value2), Some(value1));
        assert_eq!(map.get(&key), Some(&value2));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_delete_missing() {
        let mut map: HashMap<&'static str, &'static str> = HashMap::new();
        assert_eq!(map.delete(&"nonexistent"), None);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_many_collisions() {
        let mut map = HashMap::with_capacity(16);
        // Insert many keys to force collisions
        for i in 0..100 {
            let key = format!("collision_key_{}", i);
            let value = format!("value_{}", i);
            map.insert(key, value);
        }
        // Verify all are retrievable
        for i in 0..100 {
            let key = format!("collision_key_{}", i);
            let value = format!("value_{}", i);
            assert_eq!(map.get(&key), Some(&value));
        }
    }

    #[test]
    fn test_repeated_insert_delete() {
        let mut map = HashMap::with_capacity(16);

        for cycle in 0..10 {
            for i in 0..10 {
                let key = format!("cycle{}_{}", cycle, i);
                let value = format!("value{}_{}", cycle, i);
                map.insert(key, value);
            }

            for i in 0..10 {
                let key = format!("cycle{}_{}", cycle, i);
                assert!(map.delete(&key).is_some());
            }

            assert_eq!(map.len(), 0);
        }
    }

    #[test]
    fn test_zst_key() {
        let mut map = HashMap::<(), i32>::new();
        assert_eq!(map.insert((), 42), None);
        assert_eq!(map.get(&()), Some(&42));
        assert_eq!(map.insert((), 24), Some(42));
        assert_eq!(map.get(&()), Some(&24));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_zst_value() {
        let mut map = HashMap::<i32, ()>::new();
        assert_eq!(map.insert(1, ()), None);
        assert_eq!(map.insert(2, ()), None);
        assert_eq!(map.get(&1), Some(&()));
        assert_eq!(map.get(&2), Some(&()));
        assert_eq!(map.len(), 2);
    }

    #[test]
    fn test_zst_both() {
        let mut map = HashMap::<(), ()>::new();
        assert_eq!(map.insert((), ()), None);
        assert_eq!(map.get(&()), Some(&()));
        assert_eq!(map.insert((), ()), Some(()));
        assert_eq!(map.len(), 1);
    }

    #[test]
    fn test_empty_operations() {
        let mut map = HashMap::<String, String>::new();
        assert_eq!(map.get(&"nonexistent".to_string()), None);
        assert_eq!(map.delete(&"nonexistent".to_string()), None);
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn test_small_capacity() {
        let mut map = HashMap::with_capacity(1);
        let capacity = map.capacity();
        assert!(capacity.is_power_of_two()); // Should be a power of two
        assert!(capacity >= 1); // Should be at least the requested capacity

        map.insert("key", "value");
        assert_eq!(map.get(&"key"), Some(&"value"));
    }
}
