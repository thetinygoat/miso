use std::{
    fmt::{Debug, Display},
    hash::{BuildHasher, Hash, Hasher, RandomState},
};

const DEFAULT_CAPACITY: u64 = 2;
pub struct Miso<K, V>
where
    K: Hash + Clone + Debug + Display,
    V: Clone + Debug,
{
    items: Vec<Option<(K, V)>>,
    metadata: Vec<u8>,
    hash_builder: RandomState,
}

impl<K, V> Miso<K, V>
where
    K: Hash + Clone + Eq + Debug + Display,
    V: Clone + Debug,
{
    pub fn new() -> Self {
        Miso {
            items: vec![None; DEFAULT_CAPACITY as usize],
            hash_builder: RandomState::new(),
            metadata: vec![0xFF; DEFAULT_CAPACITY as usize],
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let mut idx = ((hash) & (DEFAULT_CAPACITY - 1)) as usize;
        let original_idx = idx;
        loop {
            let control = self.metadata[idx];
            // can insert
            if control == 0xFF || control == 0xFE {
                break;
            }

            let hash_fragment = control & 0x7F;
            if ((hash >> 57) & 0x7F) as u8 == hash_fragment {
                match &self.items[idx] {
                    None => break,
                    Some((existing, _)) => {
                        if key == *existing {
                            self.items[idx] = Some((key, value));
                            return;
                        }
                    }
                }
            }

            idx = (idx + 1) & (DEFAULT_CAPACITY - 1) as usize;

            if original_idx == idx {
                panic!("map full!")
            }
        }

        self.items[idx] = Some((key, value));
        self.metadata[idx] = ((hash >> 57) & 0x7F) as u8;

        println!("{:?}", self.items);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let mut hasher = self.hash_builder.build_hasher();
        key.hash(&mut hasher);
        let hash = hasher.finish();
        let idx = ((hash) & (DEFAULT_CAPACITY - 1)) as usize;
        self.items[idx].as_ref().map(|item| &item.1)
    }
}

#[cfg(test)]
mod tests {
    use crate::miso::Miso;
    #[test]
    fn test_insert() {
        let mut map = Miso::new();
        let key = String::from("key");
        let value = "value";
        map.insert(key.clone(), value);
        assert_eq!(map.get(&key), Some(&value));
    }

    #[test]
    fn test_panic() {
        let mut map = Miso::new();
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
}
