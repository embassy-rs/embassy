use heapless::LinearMap;
use heapless::linear_map::IterMut;

pub trait DynLinearMap<K, V> {
    fn get(&self, key: &K) -> Option<&V>;
    fn get_mut(&mut self, key: &K) -> Option<&mut V>;

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, None is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old value is returned.
    ///
    /// Computes in O(n) time
    fn insert(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)>;
    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    ///
    /// Computes in O(n) time
    fn remove(&mut self, key: &K) -> Option<V>;

    //    fn iter<'a>(&'a self) -> Iter<'a, K, V>;
    fn iter_mut<'a>(&'a mut self) -> IterMut<'a, K, V>;

    //    fn len(&self) -> usize;
    //    fn is_empty(&self) -> bool;
    //    fn capacity(&self) -> usize;
    //
    //    fn clear(&mut self);
}

impl<K: Eq + core::hash::Hash, V, const N: usize> DynLinearMap<K, V> for LinearMap<K, V, N> {
    fn get(&self, key: &K) -> Option<&V> {
        self.get(key)
    }

    fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        self.get_mut(key)
    }

    fn insert(&mut self, key: K, value: V) -> Result<Option<V>, (K, V)> {
        self.insert(key, value)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.remove(key)
    }

    //    fn iter<'a>(&'a self) -> Iter<'a, K, V> {
    //        self.iter()
    //    }

    fn iter_mut<'a>(&'a mut self) -> IterMut<'a, K, V> {
        self.iter_mut()
    }

    //    fn len(&self) -> usize {
    //        self.len()
    //    }
    //
    //    fn is_empty(&self) -> bool {
    //        self.is_empty()
    //    }
    //
    //    fn capacity(&self) -> usize {
    //        N
    //    }
    //
    //    fn clear(&mut self) {
    //        self.clear()
    //    }
}
