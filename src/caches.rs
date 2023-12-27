use crate::phases;

use phases::{Phase, MMSize};
use std::collections::{BTreeSet, HashMap};
use std::fmt::Debug;
use std::hash::Hash;

/// Alias for Result<T, CacheError>.
pub type CacheResult<T> = Result<T, CacheError>;
/// Manages and maintains phase entries created de
/// uma mechanismo de math.
#[derive(Default, Debug)]
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
    fn drop(&mut self, key: &K) -> CacheResult<V>;
    /// Remove all invalid entries from the cache.
    /// Returns the dropped entries. The predicate
    /// determines if an entry is valid or not
    /// where:
    /// 
    /// let (valid, invalid) == (true, false);
    fn drop_invalid(&mut self, pred: impl FnMut(&&V) -> bool) -> CacheResult<Vec<V>>;
    /// Find a match in the cache for the given
    /// key.
    fn find(&mut self, key: &K) -> CacheResult<V>;
    /// Find the closest match in the cache for
    /// the given key.
    fn find_closest(&mut self, key: &K) -> CacheResult<V>;
    /// Find a match that meets the predicate
    /// searching in reverse order.
    fn find_rev(&mut self, key: &K, pred: impl FnMut(&&V) -> bool) -> CacheResult<V>;
    /// Push a value to the cache at the given
    /// key.
    fn push(&mut self, entry: &V);
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

    fn drop(&mut self, key: &I) -> CacheResult<Self::Cached> {
        match self.find(key) {
            Ok(cached) => {
                self.entries.remove(&cached);
                self.usages.remove(&cached.input());
                Ok(cached.clone())
            },
            Err(err) => Err(err)
        }
    }
    fn drop_invalid(&mut self, mut pred: impl FnMut(&&Self::Cached) -> bool) -> CacheResult<Vec<Self::Cached>> {
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
    fn find(&mut self, key: &I) -> CacheResult<Self::Cached> { 
        self.find_rev(key, |ph| ph.input() == key)
    }
    fn find_closest(&mut self, key: &I) -> CacheResult<Self::Cached> {
        // Find the closest-- would be--
        // preceeding cached phase.
        self.find_rev(key, |ph| ph.input() <= key)
    }
    fn find_rev(&mut self, key: &I, pred: impl FnMut(&&Self::Cached) -> bool) -> CacheResult<Self::Cached> {
        let entries_clone = self.entries.clone();
        let mut iter      = entries_clone.iter().rev();

        match iter.find(pred) {
            Some(phase) => {
                self.update_usage(|_| true);
                self.usages.insert(*key, 0);
                Ok(phase.to_owned())
            },
            None => Err(CacheError::PhaseNotFound)
        }
    }
    fn push(&mut self, entry: &Self::Cached) {
        let entry_rc = entry.to_owned();
        self.usages.insert(entry_rc.input().to_owned(), 0);
        self.entries.insert(entry_rc.clone());

        // Filter out entry inputs whose usage
        // count is 0; 
        self.update_usage(|(input, _)| **input != *entry_rc.input());
    }
}

pub trait LRUCachable<I> {
    type Cached;
    /// Perform the cache entry drop algorithim.
    fn drop_invalid(&mut self) -> CacheResult<Vec<Self::Cached>>;
    /// Internal cache has too many entries.
    fn is_too_big(&mut self) -> bool;
    /// Internal cache has entries that are
    /// greater than or equal to the maximum
    /// usage age.
    fn is_too_old(&mut self) -> bool;
    /// Attempt to find a calculation phase for
    /// this machine.
    fn lookup(&mut self, n: &I) -> CacheResult<Self::Cached>;
    /// Get the maximum age an entry in the cache
    /// can reach before it becomes invalid.
    fn max_usage_age(&mut self) -> MMSize;
    /// Get the entry capacity for the internal
    /// cache.
    fn max_entry_cap(&mut self) -> MMSize;
    /// Attempt to update the cache with a
    /// calculation phase.
    fn update(&mut self, phase: &Self::Cached);
}
