use std::collections::{BTreeMap, HashMap, HashSet};
use std::hash::Hash;

pub struct Map<K, O, V> {
    values: HashMap<K, (O, V)>,
    ordered_keys: BTreeMap<O, HashSet<K>>,
}

impl<K, O, V> Map<K, O, V> {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            ordered_keys: BTreeMap::new(),
        }
    }
}

impl<K: Clone + Eq + Hash, O: Clone + Ord, V> Map<K, O, V> {
    fn remove_ordered_key(&mut self, order: &O, key: &K) {
        let keys = self.ordered_keys.get_mut(&order).unwrap();
        assert!(keys.remove(key));
        if keys.is_empty() {
            assert!(self.ordered_keys.remove(&order).is_some());
        }
    }

    /// Removes an entry by key.
    pub fn remove(&mut self, key: &K) -> Option<(O, V)> {
        let (order, value) = self.values.remove(key)?;
        self.remove_ordered_key(&order, key);
        Some((order, value))
    }

    /// Removes entries with the smallest order value.
    pub fn remove_smallest(&mut self) -> Option<(O, Vec<(K, V)>)> {
        let (order, keys) = self.ordered_keys.pop_first()?;
        let mut smallest = Vec::new();
        for key in keys {
            let (_order, value) = self.values.remove(&key).unwrap();
            smallest.push((key, value));
        }
        Some((order, smallest))
    }

    /// Returns references to entries with the smallest order value.
    pub fn peek_smallest(&self) -> Option<(&O, Vec<(&K, &V)>)> {
        let (order, keys) = self.ordered_keys.first_key_value()?;
        let mut smallest = Vec::new();
        for key in keys {
            let (_order, value) = self.values.get(&key).unwrap();
            smallest.push((key, value));
        }
        Some((order, smallest))
    }

    /// Returns the old entry with the same key if there was one.
    pub fn add(&mut self, key: K, order: O, value: V) -> Option<(O, V)> {
        let old_entry = if let Some((old_order, old_value)) = self.values.remove(&key) {
            self.remove_ordered_key(&old_order, &key);
            Some((old_order, old_value))
        } else {
            None
        };
        assert!(self
            .ordered_keys
            .entry(order.clone())
            .or_insert_with(|| HashSet::new())
            .insert(key.clone()));
        assert!(self.values.insert(key, (order, value)).is_none());
        old_entry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let mut map = Map::new();

        assert_eq!(map.add(5, 2, "a"), None);
        assert_eq!(map.ordered_keys, BTreeMap::from([(2, HashSet::from([5]))]));
        assert_eq!(map.values, HashMap::from([(5, (2, "a"))]));

        assert_eq!(map.add(5, 5, "b"), Some((2, "a")));
        assert_eq!(map.ordered_keys, BTreeMap::from([(5, HashSet::from([5]))]));
        assert_eq!(map.values, HashMap::from([(5, (5, "b"))]));

        assert_eq!(map.add(6, 2, "c"), None);
        assert_eq!(
            map.ordered_keys,
            BTreeMap::from([(5, HashSet::from([5])), (2, HashSet::from([6]))])
        );
        assert_eq!(map.values, HashMap::from([(5, (5, "b")), (6, (2, "c"))]));

        assert_eq!(map.add(7, 2, "d"), None);
        assert_eq!(
            map.ordered_keys,
            BTreeMap::from([(5, HashSet::from([5])), (2, HashSet::from([6, 7]))])
        );
        assert_eq!(
            map.values,
            HashMap::from([(5, (5, "b")), (6, (2, "c")), (7, (2, "d"))])
        );

        map.add(8, 1, "e");

        assert_eq!(map.remove(&8), Some((1, "e")));
        assert_eq!(
            map.ordered_keys,
            BTreeMap::from([(5, HashSet::from([5])), (2, HashSet::from([6, 7]))])
        );
        assert_eq!(
            map.values,
            HashMap::from([(5, (5, "b")), (6, (2, "c")), (7, (2, "d"))])
        );

        assert_eq!(map.remove(&8), None);
        assert_eq!(
            map.ordered_keys,
            BTreeMap::from([(5, HashSet::from([5])), (2, HashSet::from([6, 7]))])
        );
        assert_eq!(
            map.values,
            HashMap::from([(5, (5, "b")), (6, (2, "c")), (7, (2, "d"))])
        );

        assert_eq!(
            map.peek_smallest(),
            Some((&2, vec![(&6, &"c"), (&7, &"d")]))
        );

        assert_eq!(map.remove_smallest(), Some((2, vec![(6, "c"), (7, "d")])));
        assert_eq!(map.ordered_keys, BTreeMap::from([(5, HashSet::from([5]))]));
        assert_eq!(map.values, HashMap::from([(5, (5, "b"))]));

        assert_eq!(map.remove_smallest(), Some((5, vec![(5, "b")])));

        assert!(map.values.is_empty());
        assert!(map.ordered_keys.is_empty());

        assert_eq!(map.peek_smallest(), None);
        assert_eq!(map.remove_smallest(), None);
    }
}
