use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::mem;
use std::borrow::Borrow;

const INITIAL_BUCKETS: usize = 1;

pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    /// number of items in the hash-map (for easy access)
    items: usize,
}

impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            // allocation happens during initial insert.
            buckets: Vec::new(),
            items: 0,
        }
    }
}

impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    /// We need K and Q to have implementations of the Hash and Eq traits that produce identical results
    fn bucket<Q>(&self, key: &Q) -> Option<usize>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        if self.buckets.is_empty() {
            return None;
        }
        // need to create a new hasher everytime for a fresh hash value.
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        // TODO: Implement something better than modulo
        Some((hasher.finish() % self.buckets.len() as u64) as usize)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key, `None` is returned. If the map did have this key
    /// present, the value is updated, and the old value is returned.
    ///
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        // check if resize is needed
        if self.buckets.is_empty() || self.items > 3 * self.buckets.len() / 4 {
            self.resize();
        }

        let bucket = self.bucket(&key).expect("");
        let bucket: &mut Vec<(K, V)> = &mut self.buckets[bucket];

        // `&mut` in pattern matching dereferences the tuple it gets from the iterator
        // with `ref`, ekey is borrowed instead of moved in the pattern.
        // with `ref mut`, evalue is borrowed mutably instead of moved in the pattern.
        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value));
            }
        }

        self.items += 1;
        bucket.push((key, value));
        None
    }

    /// Returns a reference to the value corresponding to the key.
    /// K can be borrowed as Q, so that you don't always have to provide a reference to a K
    pub fn get<Q>(&self, key: &Q) -> Option<&V> 
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key)?;
        self.buckets[bucket]
            .iter()
            .find(|&(ref ekey, _)| ekey.borrow() == key)
            .map(|&(_, ref v)| v)
    }

    /// Returns true if the key is in the map, false otherwise.
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        self.get(key).is_some()
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the
    /// map.
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let bucket = self.bucket(key)?;
        let bucket = &mut self.buckets[bucket];
        let i = bucket.iter().position(|&(ref ekey, _)| ekey.borrow() == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(i).1)
    }

    /// Returns the number of items that are currently in the map.
    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => INITIAL_BUCKETS,
            n => 2 * n,
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));

        // so expensive!!
        for (key, value) in self
            .buckets
            .iter_mut()
            .flat_map(|bucket| bucket.drain(..))
        {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((key, value));
        }

        mem::replace(&mut self.buckets, new_buckets);
    }
}

pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => {
                    match bucket.get(self.at) {
                        Some(&(ref k, ref v)) => {
                            self.at += 1;
                            break Some((k, v));
                        },
                        // no more items in the bucket, move to next bucket
                        None => {
                            self.bucket += 1;
                            self.at = 0;
                            continue;
                        },
                    }
                },
                None => break None,
            }
        }
    }
}

impl<'a, K, V> IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            bucket: 0,
            at: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert() {
        let mut map = HashMap::new();

        assert!(map.is_empty());
        assert_eq!(map.len(), 0);

        map.insert("foo", 42);

        assert!(!map.is_empty());
        assert_eq!(map.len(), 1);

        assert_eq!(map.get(&"foo"), Some(&42));
        assert_eq!(map.remove(&"foo"), Some(42));
        assert_eq!(map.get(&"foo"), None);

        assert!(map.is_empty());
        assert_eq!(map.len(), 0);
    }

    #[test]
    fn iter() {
        let mut map = HashMap::new();
        map.insert("foo", 42);
        map.insert("bar", 43);
        map.insert("baz", 44);
        map.insert("quo", 45);
        for (&k, &v) in &map {
            match k {
                "foo" => assert_eq!(v, 42),
                "bar" => assert_eq!(v, 43),
                "baz" => assert_eq!(v, 44),
                "quo" => assert_eq!(v, 45),
                _ => unreachable!(),
            }
        }
        assert_eq!((&map).into_iter().count(), 4);
    }

    #[test]
    fn empty_hashmap() {
        let mut map = HashMap::<String, String>::new();
        assert_eq!(map.get("key"), None);
        assert_eq!(map.contains_key("key"), false);
        assert_eq!(map.remove("key"), None);
    }
}
