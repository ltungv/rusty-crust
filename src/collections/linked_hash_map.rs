use std::borrow::Borrow;
use std::collections::hash_map::RandomState;
use std::hash::{BuildHasher, Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Index;

/// A basic hash map.
///
/// It is required that the keys implement the [`Eq`] and [`Hash`] traits,
/// although this can frequently be achieved by using
/// `#[derive(PartialEq, Eq, Hash)]`. If you implement these yourself, it is
/// important that the following property holds:
///
/// ```text
/// k1 == k2 -> hash(k1) == hash(k2)
/// ```
///
/// In other words, if two keys are equal, their hashes must be equal.
///
/// # Attributions
///
/// This `LinkedHashMap` implementation is based off [Jon Gjengset's livestream]
/// on the concept and implementation of the data structure itself. The
/// [source code] of the project from the livestream can be found on Github.
///
/// [Jon Gjengset's livestream]: https://www.youtube.com/watch?v=k6xR2kf9hlA
/// [source code]: https://github.com/jonhoo/rust-basic-hashmap
///
/// # Examples
///
/// These examples are taken from Rust's standard library.
///
/// ```
/// use dt::containers::LinkedHashMap;
///
/// // Type inference lets us omit an explicit type signature (which
/// // would be `LinkedHashMap<String, String>` in this example).
/// let mut book_reviews = LinkedHashMap::new();
///
/// // Review some books.
/// book_reviews.insert(
///     "Adventures of Huckleberry Finn".to_string(),
///     "My favorite book.".to_string(),
/// );
/// book_reviews.insert(
///     "Grimms' Fairy Tales".to_string(),
///     "Masterpiece.".to_string(),
/// );
/// book_reviews.insert(
///     "Pride and Prejudice".to_string(),
///     "Very enjoyable.".to_string(),
/// );
/// book_reviews.insert(
///     "The Adventures of Sherlock Holmes".to_string(),
///     "Eye lyked it alot.".to_string(),
/// );
///
/// // Check for a specific one.
/// // When containers store owned values (String), they can still be
/// // queried using references (&str).
/// if !book_reviews.contains_key("Les Misérables") {
///     println!("We've got {} reviews, but Les Misérables ain't one.",
///              book_reviews.len());
/// }
///
/// // oops, this review has a lot of spelling mistakes, let's delete it.
/// book_reviews.remove("The Adventures of Sherlock Holmes");
///
/// // Look up the values associated with some keys.
/// let to_find = ["Pride and Prejudice", "Alice's Adventure in Wonderland"];
/// for &book in &to_find {
///     match book_reviews.get(book) {
///         Some(review) => println!("{}: {}", book, review),
///         None => println!("{} is unreviewed.", book)
///     }
/// }
///
/// // Look up the value for a key (will panic if the key is not found).
/// println!("Review for Jane: {}", book_reviews["Pride and Prejudice"]);
///
/// // Iterate over everything.
/// for (book, review) in &book_reviews {
///     println!("{}: \"{}\"", book, review);
/// }
/// ```
///
/// The easiest way to use `LinkedHashMap` with a custom key type is to derive
/// [`Eq`] and [`Hash`].
/// We must also derive [`PartialEq`].
///
/// ```
/// use dt::containers::LinkedHashMap;
///
/// #[derive(Hash, Eq, PartialEq, Debug)]
/// struct Viking {
///     name: String,
///     country: String,
/// }
///
/// impl Viking {
///     /// Creates a new Viking.
///     fn new(name: &str, country: &str) -> Viking {
///         Viking { name: name.to_string(), country: country.to_string() }
///     }
/// }
///
/// // Use a LinkedHashMap to store the vikings' health points.
/// let mut vikings = LinkedHashMap::new();
///
/// vikings.insert(Viking::new("Einar", "Norway"), 25);
/// vikings.insert(Viking::new("Olaf", "Denmark"), 24);
/// vikings.insert(Viking::new("Harald", "Iceland"), 12);
///
/// // Use derived implementation to print the status of the vikings.
/// for (viking, health) in &vikings {
///     println!("{:?} has {} hp", viking, health);
/// }
/// ```
///
/// LinkedHashMap also implements an Entry API, which allows for more complex
/// methods of getting, setting, updating and removing keys and their values:
///
/// ```
/// use dt::containers::LinkedHashMap;
///
/// // type inference lets us omit an explicit type signature (which
/// // would be `LinkedHashMap<&str, u8>` in this example).
/// let mut player_stats = LinkedHashMap::new();
///
/// fn random_stat_buff() -> u8 {
///     // could actually return some random value here - let's just return
///     // some fixed value for now
///     42
/// }
///
/// // insert a key only if it doesn't already exist
/// player_stats.entry("health").or_insert(100);
///
/// // insert a key using a function that provides a new value only if it
/// // doesn't already exist
/// player_stats.entry("defence").or_insert_with(random_stat_buff);
///
/// // update a key, guarding against the key possibly not being set
/// let stat = player_stats.entry("attack").or_insert(100);
/// *stat += random_stat_buff();
/// ```
///
/// A LinkedHashMap with fixed list of elements can be initialized from an
/// array.
///
/// ```
/// use dt::containers::LinkedHashMap;
///
/// let timber_resources: LinkedHashMap<&str, i32> =
///     [("Norway", 100), ("Denmark", 50), ("Iceland", 10)]
///     .iter().cloned().collect();
/// // use the values stored in map
/// ```
#[derive(Debug)]
pub struct LinkedHashMap<K, V, S = RandomState> {
    // This hash map implementation relies on an array of buckets that is
    // indexed by the hash of an entry's key. If 2 different keys are hashed to
    // the same value, the entries are put into the same bucket. These entries
    // can later be retrieved by comparing both the hashed key and the actual
    // key.
    buckets: Vec<Bucket<K, V>>,
    hasher_builder: S,
    entries_count: usize,
}

/// A data item that holds entries in [`LinkedHashMap`] whose key is hashed to
/// the same value.
///
/// [`LinkedHashMap`]: crate::containers::LinkedHashMap
#[derive(Debug)]
struct Bucket<K, V> {
    items: Vec<(K, V)>,
}

impl<K, V> Default for Bucket<K, V> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

/// Deriving the bucket's index from the `hashable` value.
fn derive_bucket_index<H, K>(mut hasher: H, key: &K, n_buckets: usize) -> usize
where
    H: Hasher,
    K: Hash + ?Sized,
{
    key.hash(&mut hasher);
    (hasher.finish() % n_buckets as u64) as usize
}

impl<K, V> Default for LinkedHashMap<K, V, RandomState> {
    fn default() -> Self {
        Self {
            buckets: Vec::new(),
            hasher_builder: RandomState::new(),
            entries_count: 0,
        }
    }
}

impl<K, V> LinkedHashMap<K, V, RandomState> {
    /// Creates an empty `LinkedHashMap`.
    ///
    /// The hash map is initially created with an empty list of buckets, so it
    /// will not allocate until it is first inserted into.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::LinkedHashMap;
    /// let mut map: LinkedHashMap<&str, i32> = LinkedHashMap::new();
    /// ```
    pub fn new() -> Self {
        Default::default()
    }
}

impl<K, V, S> LinkedHashMap<K, V, S> {
    /// Returns the number of elements in the map.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::LinkedHashMap;
    ///
    /// let mut a = LinkedHashMap::new();
    /// assert_eq!(a.len(), 0);
    /// a.insert(1, "a");
    /// assert_eq!(a.len(), 1);
    /// ```
    pub fn len(&self) -> usize {
        self.entries_count
    }

    /// Returns true if the map contains no elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::LinkedHashMap;
    ///
    /// let mut a = LinkedHashMap::new();
    /// assert!(a.is_empty());
    /// a.insert(1, "a");
    /// assert!(!a.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.entries_count == 0
    }
}

impl<K, V, S> LinkedHashMap<K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, [`None`] is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old 
    /// value is returned.  The key is not updated, though; this matters for 
    /// types that can be `==` without being identical.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::LinkedHashMap;
    ///
    /// let mut map = LinkedHashMap::new();
    /// assert_eq!(map.insert(37, "a"), None);
    /// assert_eq!(map.is_empty(), false);
    ///
    /// map.insert(37, "b");
    /// assert_eq!(map.insert(37, "c"), Some("b"));
    /// assert_eq!(map[&37], "c");
    /// ```
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty()
            || self.entries_count > 3 * self.buckets.len() / 4
        {
            self.grow();
        }

        let idx = self.index(&key);
        let bucket = &mut self.buckets[idx];

        for &mut (ref k, ref mut v) in bucket.items.iter_mut() {
            if *k == key {
                return Some(std::mem::replace(v, value));
            }
        }
        bucket.items.push((key, value));
        self.entries_count += 1;
        None
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map’s key type, but Hash and Eq 
    /// on the borrowed form must match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::LinkedHashMap;
    /// let mut map = LinkedHashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.get(&1), Some(&"a"));
    /// assert_eq!(map.get(&2), None);
    /// ```
    pub fn get<Q>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let idx = self.index(key);
        self.buckets[idx]
            .items
            .iter()
            .find(|(k, _)| k.borrow() == key)
            .map(|(_, v)| v)
    }

    /// Removes a key from the map, returning the value at the key if the key 
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map’s key type, but Hash and Eq 
    /// on the borrowed form must match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::LinkedHashMap;
    ///
    /// let mut map = LinkedHashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.remove(&1), Some("a"));
    /// assert_eq!(map.remove(&1), None);
    /// ```
    pub fn remove<Q>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let idx = self.index(key);
        let bucket = &mut self.buckets[idx];

        let entry_idx = bucket
            .items
            .iter()
            .position(|(k, _)| k.borrow() == key)?;
        self.entries_count -= 1;
        Some(bucket.items.swap_remove(entry_idx).1)
    }

    /// Gets the given key’s corresponding entry in the map for in-place 
    /// manipulation.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// let mut letters = HashMap::new();
    ///
    /// for ch in "a short treatise on fungi".chars() {
    ///     let counter = letters.entry(ch).or_insert(0);
    ///     *counter += 1;
    /// }
    ///
    /// assert_eq!(letters[&'s'], 2);
    /// assert_eq!(letters[&'t'], 3);
    /// assert_eq!(letters[&'u'], 1);
    /// assert_eq!(letters.get(&'y'), None);
    /// ```
    pub fn entry(&mut self, key: K) -> Entry<'_, K, V, S> {
        if self.buckets.is_empty()
            || self.entries_count > 3 * self.buckets.len() / 4
        {
            self.grow();
        }

        let bucket_idx = self.index(&key);
        if let Some(entry_idx) = self.buckets[bucket_idx]
            .items
            .iter_mut()
            .position(|&mut (ref k, _)| *k == key)
        {
            // We are using `position` instead of `find` cause `self.buckets` 
            // will be borrowed inside the scope of the if statement; using 
            // `find` somehow scoped `self.buckets` within the function, 
            // disallowing `self` to be re-borrowed in the else case.
            let &mut (ref key, ref mut value) =
                &mut self.buckets[bucket_idx].items[entry_idx];
            return Entry::Occupied(OccupiedEntry { key, value });
        }
        Entry::Vacant(VacantEntry {
            key,
            bucket_idx,
            map: self,
        })
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map’s key type, but Hash and Eq 
    /// on the borrowed form must match those for the key type.
    ///
    /// # Examples
    ///
    /// ```
    /// use dt::containers::LinkedHashMap;
    ///
    /// let mut map = LinkedHashMap::new();
    /// map.insert(1, "a");
    /// assert_eq!(map.contains_key(&1), true);
    /// assert_eq!(map.contains_key(&2), false);
    /// ```
    pub fn contains_key<Q>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        let idx = self.index(key);
        self.buckets[idx]
            .items
            .iter()
            .any(|(k, _)| k.borrow() == key)
    }

    /// Increase the size of the array of buckets. If there is no bucket, extend 
    /// the array by one, otherwise, double the array's size and reindex all 
    /// existing entries.
    fn grow(&mut self) {
        let target_size = match self.buckets.len() {
            0 => 1,
            n => 2 * n,
        };

        let mut buckets = Vec::with_capacity(target_size);
        buckets.extend((0..target_size).map(|_| Bucket::default()));
        for (key, value) in self
            .buckets
            .iter_mut()
            .flat_map(|bucket| bucket.items.drain(..))
        {
            let idx = derive_bucket_index(
                self.hasher_builder.build_hasher(),
                &key,
                target_size,
            );
            buckets[idx].items.push((key, value));
        }
        self.buckets = buckets;
    }

    /// Get the index of the bucket for `key`.
    fn index<Q>(&self, key: &Q) -> usize
    where
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized,
    {
        derive_bucket_index(
            self.hasher_builder.build_hasher(),
            key,
            self.buckets.len(),
        )
    }
}

impl<K, Q, V, S> Index<&Q> for LinkedHashMap<K, V, S>
where
    K: Hash + Eq + Borrow<Q>,
    Q: Hash + Eq + ?Sized,
    S: BuildHasher,
{
    type Output = V;

    fn index(&self, key: &Q) -> &Self::Output {
        self.get(key).unwrap()
    }
}

impl<K, V> FromIterator<(K, V)> for LinkedHashMap<K, V, RandomState>
where
    K: Hash + Eq,
{
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        let mut map = Self::new();
        for (k, v) in iter {
            map.insert(k, v);
        }
        map
    }
}

/// An iterator over the elements of a [`LinkedHashMap`].
///
/// [`LinkedHashMap`]: crate::containers::LinkedHashMap
#[derive(Debug)]
pub struct Iter<'a, K, V, S> {
    map: &'a LinkedHashMap<K, V, S>,
    bucket_idx: usize,
    bucket_entry_idx: usize,
}

impl<'a, K, V, S> Iterator for Iter<'a, K, V, S> {
    type Item = (&'a K, &'a V);

    /// We keep two indices, one index for the bucket and one index for the 
    /// entry within the bucket that is currently pointed at.
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(bucket) = self.map.buckets.get(self.bucket_idx) {
            if let Some((key, value)) =
                bucket.items.get(self.bucket_entry_idx)
            {
                self.bucket_entry_idx += 1;
                return Some((key, value));
            }
            self.bucket_idx += 1;
            self.bucket_entry_idx = 0;
        }
        None
    }
}

impl<'a, K, V, S> IntoIterator for &'a LinkedHashMap<K, V, S> {
    type Item = (&'a K, &'a V);

    type IntoIter = Iter<'a, K, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            map: self,
            bucket_idx: 0,
            bucket_entry_idx: 0,
        }
    }
}

/// An iterator over the elements of a [`LinkedHashMap`].
///
/// [`LinkedHashMap`]: crate::containers::LinkedHashMap
#[derive(Debug)]
pub struct IntoIter<K, V, S> {
    map: LinkedHashMap<K, V, S>,
    bucket_idx: usize,
}

impl<K, V, S> Iterator for IntoIter<K, V, S> {
    type Item = (K, V);

    /// We keep two indices, one index for the bucket and one index for the 
    /// entry within the bucket that is currently pointed at.
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(bucket) = self.map.buckets.get_mut(self.bucket_idx) {
            if let Some((key, value)) = bucket.items.pop() {
                return Some((key, value));
            }
            self.bucket_idx += 1;
        }
        None
    }
}

impl<K, V, S> IntoIterator for LinkedHashMap<K, V, S> {
    type Item = (K, V);

    type IntoIter = IntoIter<K, V, S>;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            map: self,
            bucket_idx: 0,
        }
    }
}

#[derive(Debug)]
pub struct OccupiedEntry<'a, K, V> {
    key: &'a K,
    value: &'a mut V,
}

#[derive(Debug)]
pub struct VacantEntry<'a, K, V, S> {
    key: K,
    bucket_idx: usize,
    map: &'a mut LinkedHashMap<K, V, S>,
}

#[derive(Debug)]
pub enum Entry<'a, K, V, S> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V, S>),
}

impl<'a, K, V, S> Entry<'a, K, V, S> {
    pub fn key(&self) -> &K {
        match *self {
            Self::Occupied(OccupiedEntry { key, value: _ }) => key,
            Self::Vacant(VacantEntry {
                ref key,
                bucket_idx: _,
                map: _,
            }) => key,
        }
    }

    pub fn and_modify<F>(self, f: F) -> Self
    where
        F: FnOnce(&mut V),
    {
        if let Self::Occupied(occupied_entry) = self {
            f(occupied_entry.value);
            return Self::Occupied(occupied_entry);
        }
        self
    }
}

impl<'a, K, V, S> Entry<'a, K, V, S>
where
    K: Hash + Eq,
    S: BuildHasher,
{
    pub fn or_default(self) -> &'a mut V
    where
        V: Default,
    {
        self.or_insert(Default::default())
    }

    pub fn or_insert(self, value: V) -> &'a mut V {
        match self {
            Self::Occupied(OccupiedEntry { key: _, value }) => value,
            Self::Vacant(VacantEntry {
                key,
                bucket_idx,
                map,
            }) => {
                map.insert(key, value);
                // unwrap() cause we just inserted a value
                let &mut (_, ref mut value) =
                    map.buckets[bucket_idx].items.last_mut().unwrap();
                value
            }
        }
    }

    pub fn or_insert_with<F>(self, f: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        self.or_insert(f())
    }

    pub fn or_insert_with_key<F>(self, f: F) -> &'a mut V
    where
        F: FnOnce(&K) -> V,
    {
        match self {
            Self::Occupied(OccupiedEntry { key: _, value }) => value,
            Self::Vacant(VacantEntry {
                key,
                bucket_idx,
                map,
            }) => {
                let value = f(&key);
                map.insert(key, value);
                // unwrap() cause we just inserted a value
                let &mut (_, ref mut value) =
                    map.buckets[bucket_idx].items.last_mut().unwrap();
                value
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn basic_crud() {
        let mut map = LinkedHashMap::new();
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        // Test creation.
        map.insert("foo", 42);
        assert_eq!(map.get(&"foo"), Some(&42));
        assert_eq!(map[&"foo"], 42);
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());

        // Test update.
        map.insert("foo", 43);
        assert_eq!(map.get(&"foo"), Some(&43));
        assert_eq!(map.len(), 1);
        assert!(!map.is_empty());

        // Test removal.
        assert_eq!(map.remove(&"foo"), Some(43));
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());

        // Test operations on a non-existent key.
        assert_eq!(map.get(&"foo"), None);
        assert_eq!(map.remove(&"foo"), None);
        assert_eq!(map.len(), 0);
        assert!(map.is_empty());
    }

    #[test]
    fn iterator() {
        // Keys and values to insert. Keys must be pair-wise different.
        let test_vals: HashMap<_, _> =
            vec![("foo", 7), ("bar", 11), ("baz", 13), ("quox", 17)]
                .into_iter()
                .collect();

        // Keep track of whether an entry has been seen.
        let mut has_seen: HashMap<_, _> = vec![
            ("foo", false),
            ("bar", false),
            ("baz", false),
            ("quox", false),
        ]
        .into_iter()
        .collect();

        let mut map = LinkedHashMap::new();
        for (&k, &v) in &test_vals {
            map.insert(k, v);
        }

        for (&k, &v) in &map {
            let expected = test_vals.get(k).cloned().unwrap();
            // Check if the iterator returns the correct item.
            assert_eq!(v, expected);
            // Check if some key is returned once again.
            assert_eq!(has_seen.insert(k, true), Some(false));
        }
        // Check if the iterator has gone through all items.
        assert!(has_seen.iter().all(|(_, &v)| v));

        has_seen.iter_mut().for_each(|(_, v)| *v = false);
        for (k, v) in map {
            let expected = test_vals.get(k).cloned().unwrap();
            // Check if the iterator returns the correct item.
            assert_eq!(v, expected);
            // Check if some key is returned once again.
            assert_eq!(has_seen.insert(k, true), Some(false));
        }
        // Check if the iterator has gone through all items.
        assert!(has_seen.iter().all(|(_, &v)| v));
    }
}

