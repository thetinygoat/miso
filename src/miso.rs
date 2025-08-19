use std::{
    fmt::Debug,
    hash::{BuildHasher, Hash, Hasher, RandomState},
};

const DEFAULT_CAPACITY: usize = 1024;
const EMPTY: u8 = 0x80;
const DELETED: u8 = 0xFE;
pub struct Miso<K, V> {
    items: Vec<Option<(K, V)>>,
    metadata: Vec<u8>,
    hash_builder: RandomState,
    capacity: usize,
    size: usize,
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
    K: Hash + Clone + Eq + Debug,
    V: Clone + Debug,
{
    pub fn new() -> Self {
        Miso {
            items: vec![None; DEFAULT_CAPACITY],
            hash_builder: RandomState::new(),
            metadata: vec![EMPTY; DEFAULT_CAPACITY],
            capacity: DEFAULT_CAPACITY,
            size: 0,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let capacity = capacity.next_power_of_two();
        Miso {
            items: vec![None; capacity],
            hash_builder: RandomState::new(),
            metadata: vec![EMPTY; capacity],
            capacity,
            size: 0,
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        match self.probe_for_insert(hash, &key) {
            Some((index, new)) => {
                self.items[index] = Some((key, value));
                let hash_fingerprint = ((hash >> 57) & (0x7F)) as u8;
                self.metadata[index] = hash_fingerprint;
                if new {
                    self.size += 1;
                }
            }
            None => {
                // todo: resize?
                panic!("table full!")
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
        let mut index = ((hash) & (self.capacity - 1) as u64) as usize;
        let original_index = index;
        let mut reusable_index = None;
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
            let hash_fingerprint = ((hash >> 57) & 0x7F) as u8;

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
                self.metadata[index] = DELETED;
                return item;
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::miso::Miso;
    #[test]
    fn test_insert() {
        let mut map = Miso::new();
        let key = "key";
        let value = "value";
        map.insert(key, value);
        assert_eq!(map.get(&key), Some(&value));
        assert_eq!(map.size(), 1);
    }

    #[test]
    #[should_panic]
    fn test_full_table() {
        let mut map = Miso::with_capacity(2);
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
        assert_eq!(map.get(&key1), Some(&value1));
        assert_eq!(map.get(&key1), Some(&value1));
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
}
