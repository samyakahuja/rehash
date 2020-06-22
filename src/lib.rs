use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const INITIAL_BUCKETS: usize = 1;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            // allocation happens during initial insert.
            buckets: Vec::new(),
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key, `None` is returned. If the map did have this key
    /// present, the value is updated, and the old value is returned.
    ///
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // need to create a new hasher everytime for a fresh hash value.
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        let bucket = (hasher.finish() % self.buckets.len() as u64) as usize;
        let bucket: &mut Vec<(K, V)> = &mut self.buckets[bucket];

        // `&mut` in pattern matching dereferences the tuple it gets from the iterator
        // with `ref`, ekey is borrowed instead of moved in the pattern.
        // with `ref mut`, evalue is borrowed mutably instead of moved in the pattern.
        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                use std::mem;
                return Some(mem::replace(evalue, value));
            }
        }

        bucket.push((key, value));
        None
    }

    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_BUCKETS,
            n => 2 * n,
        };

        // TODO: resize
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = HashMap::new();
        map.insert("foo", 42);
    }
}
