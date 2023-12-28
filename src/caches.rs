use crate::phases;

use phases::{Phase, MMSize};
use std::collections::{BTreeSet, HashMap};
use std::fmt::Debug;
use std::hash::Hash;

/// Alias for Result<T, CacheError>.
pub type CacheResult<T> = Result<T, CacheError>;
/// Test cache. Used only for testing purposes.
/// Implements the basic needs of the Caches
/// interface for the test case listed below.
///
/// ```
/// use math_machines::caches::{Caches, TestCache};
///
/// let mut cache = TestCache::default();
/// cache.push(8);
/// cache.push(5);
///
/// let value = cache.find(8).expect("an integer");
/// assert_eq!(value, 8);
///
/// let value = cache.find(5).expect("an integer");
/// assert_eq!(value, 5);
///
/// assert_eq!(cache.entries.len(), 2);
/// assert_eq!(cache.usages.len(), 2);
/// ```
#[derive(Default, Debug)]
pub struct TestCache {
    pub entries: BTreeSet<u8>,
    pub usages:  HashMap<u8, usize>
}
/// Manages and maintains phase entries created de
/// uma mechanismo de math.
///
/// ```
/// use math_machines::{Caches, MachineCache};
/// use math_machines::phases::{Newable, Phase};
/// let mut cache = MachineCache::<u8, u8>::new();
///
/// let mut phase1 = Phase::<u8, u8>::new();
/// phase1.setinput(8);
/// cache.push(phase1.clone());
///
/// let mut phase2 = Phase::<u8, u8>::new();
/// phase2.setinput(16);
/// cache.push(phase2.clone());
///
/// cache.find(*phase2.input());
///
/// let mut phase3 = Phase::<u8, u8>::new();
/// phase3.setinput(44);
/// cache.push(phase3.clone());
///
/// assert_eq!(cache.len(), 3);
/// assert_eq!(cache.highest_usage(), 3);
///
/// let found = cache.find(*phase1.input()).expect("calculation phase");
/// assert_eq!(*found.input(), 8);
///
/// let found = cache.find_closest(17).expect("calculation phase");
/// assert_eq!(*found.input(), 16);
///
/// assert_eq!(cache.highest_usage(), 2);
/// ```
#[derive(Clone, Debug)]
pub struct MachineCache<T, I> {
    /// Actual cache entries of `Phase` objects.
    entries: BTreeSet<Phase<T, I>>,
    /// Tracks usage count per entry N of the
    /// cache.
    usages:  HashMap<I, MMSize>,
}
/// Error occurred during the manipulation,
/// retrieval from/updating into a cache, or
/// directly in, a `Phase`.
#[derive(Debug)]
pub enum CacheError {
    /// Phase could not be found in a cache or
    /// other collection.
    PhaseNotFound,
}

/// A type can act as a cache for some other data
/// type.
pub trait Caches<K: Hash + ?Sized, V: Sized> {
    type Cached;
    /// Remove an entry at the key from the cache
    /// returning the value, if it exists.
    fn drop(&mut self, key: K) -> CacheResult<V>;
    /// Remove all invalid entries from the cache.
    /// Returns the dropped entries. The predicate
    /// determines if an entry is valid or not
    /// where:
    /// 
    /// let (valid, invalid) == (true, false);
    fn drop_invalid(&mut self, pred: impl FnMut(&V) -> bool) -> CacheResult<Vec<V>>;
    /// Find a match in the cache for the given
    /// key.
    fn find(&mut self, key: K) -> CacheResult<V>;
    /// Find the closest match in the cache for
    /// the given key.
    fn find_closest(&mut self, key: K) -> CacheResult<V>;
    /// Find a match that meets the predicate
    /// searching in reverse order.
    fn find_rev(&mut self, pred: impl FnMut(&&V) -> bool) -> CacheResult<V>;
    /// Push a value to the cache at the given
    /// key.
    fn push(&mut self, entry: V);
}

impl Caches<u8, u8> for TestCache {
    type Cached = u8;
    fn drop(&mut self, key: u8) -> CacheResult<u8> {
        match self.find(key) {
            Ok(cached) => {
                self.entries.remove(&cached);
                self.usages.remove(&cached);
                Ok(cached.clone())
            },
            Err(err) => Err(err)
        }
    }
    fn drop_invalid(&mut self, _: impl FnMut(&u8) -> bool) -> CacheResult<Vec<u8>> {
        let mut retn = vec![];
        let entries_clone = self.entries.clone();
        let mut iter      = entries_clone.iter().rev();

        while let Some(cached) = iter.next() {
            if true {
                continue;
            }
            retn.push(cached.to_owned());
            self.entries.remove(cached);
            self.usages.remove(&cached);
        }
        Ok(retn)
    }
    fn find(&mut self, key: u8) -> CacheResult<u8> {
        self.find_rev(|cached| **cached == key)
    }
    fn find_closest(&mut self, key: u8) -> CacheResult<u8> {
        // Find the closest-- would be--
        // preceeding cached phase.
        self.find_rev(|cached| **cached <= key)
    }
    fn find_rev(&mut self, pred: impl FnMut(&&u8) -> bool) -> CacheResult<u8> {
        let entries_clone = self.entries.clone();
        let mut iter      = entries_clone.iter().rev();

        match iter.find(pred) {
            Some(phase) => {
                println!("{phase}");
                self.usages.insert(*phase, 0);
                Ok(phase.to_owned())
            },
            None => Err(CacheError::PhaseNotFound)
        }
    }
    fn push(&mut self, entry: u8) {
        self.usages.insert(entry.clone(), 0);
        self.entries.insert(entry.clone());
    }
}

impl<T: Sized, I> MachineCache<T, I>
where
    I: Hash + Sized + Clone + Copy + Debug + PartialEq + Eq,
{
    /// Return the greatest count of iterations
    /// since last visit/use of any value in this
    /// cache.
    pub fn highest_usage(&self) -> MMSize {
        let mut us: Vec<&MMSize> = self.usages.values().collect();
        us.sort();
        **us.last().unwrap_or(&&0)
    }
    /// Number of entries in this cache.
    pub fn len(&self) -> MMSize {
        self.entries.len()
    }
    pub fn new() -> Self {
        Self {
            entries: BTreeSet::new(),
            usages:  HashMap::new()
        }
    }
    /// Update the usage of individual entry
    /// usages.
    fn update_usage(&mut self, filt: impl FnMut(&(&I, &MMSize)) -> bool) {
        let usage_clone = self.usages.clone();
        let mut iter = usage_clone.iter().filter(filt);
        // Iterate through the usages, after push
        // of new data, to increment usage by 1
        // for each usage record.
        while let Some((input, usage)) = iter.next() {
            self.usages.insert(*input, usage+1);
        }
    }
    /// Validator to ensure the usage of a value
    /// is less than the oldest in usages map.
    fn valid_usage(&self, key: &I) -> bool {
        let key_usage = self.usages.get(key).expect("usage count");
        let gts_usage = &self.highest_usage();
        key_usage < gts_usage
    }
}

// Going off pattern by implementing Caches traits
// here to hopefully better illustrate usage
// specifically for a MathMachine `MachineCache`.
impl<T, I> Caches<I, Phase<T, I>> for MachineCache<T, I>
where
    I: Clone + Copy + Debug + Default + Eq + Hash + Ord + PartialEq + Sized,
    T: Clone + Debug + Default + Ord + Sized,
{
    type Cached = Phase<T, I>;

    fn drop(&mut self, key: I) -> CacheResult<Self::Cached> {
        match self.find(key) {
            Ok(cached) => {
                self.entries.remove(&cached);
                self.usages.remove(&cached.input());
                Ok(cached.clone())
            },
            Err(err) => Err(err)
        }
    }
    fn drop_invalid(&mut self, mut pred: impl FnMut(&Self::Cached) -> bool) -> CacheResult<Vec<Self::Cached>> {
        let mut retn = vec![];
        let entries_clone = self.entries.clone();
        let mut iter      = entries_clone.iter().rev();

        while let Some(p) = iter.next() {
            if pred(&p) && self.valid_usage(&p.input()) {
                continue;
            }
            retn.push(p.to_owned());
            self.entries.remove(p);
            self.usages.remove(&p.input());
        }
        Ok(retn)
    }
    fn find(&mut self, key: I) -> CacheResult<Self::Cached> { 
        self.find_rev(|ph| *ph.input() == key)
    }
    fn find_closest(&mut self, key: I) -> CacheResult<Self::Cached> {
        // Find the closest-- would be--
        // preceeding cached phase.
        self.find_rev(|ph| ph.input() <= &key)
    }
    fn find_rev(&mut self, pred: impl FnMut(&&Self::Cached) -> bool) -> CacheResult<Self::Cached> {
        let entries_clone = self.entries.clone();
        let mut iter      = entries_clone.iter().rev();

        match iter.find(pred) {
            Some(phase) => {
                self.update_usage(|_| true);
                self.usages.insert(*phase.input(), 0);
                Ok(phase.to_owned())
            },
            None => Err(CacheError::PhaseNotFound)
        }
    }
    fn push(&mut self, entry: Self::Cached) {
        self.entries.insert(entry.clone());
        self.usages.insert(*entry.input(), 0);

        // Filter out entry inputs whose usage
        // count is 0;
        self.update_usage(|(input, _)| **input != *entry.input());
    }
}
