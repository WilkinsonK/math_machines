use std::collections::{BTreeSet, HashMap};
use std::borrow::Borrow;
use std::hash::Hash;
use std::rc::Rc;

const PHASE_SIZE: MMSize = 4;
const PHASE_ARGC: MMSize = 1;

/// Variable type alias for the size of integer
/// math machines use.
pub type MMInt = u128;
/// Variable type alias for the `size` type
/// math machines use.
pub type MMSize = usize;
/// A slice of values used to calculate some
/// result. Phases are processed by caches to make
/// calculation of large numbers faster. 0th and
/// 1st values are reserved for the result of the
/// phase where remaining `MMNumeric`s
/// `(2nd, 3rd, 4th, ...)` are the arguments to
/// achieve said result.
pub type Phase = [MMInt; PHASE_SIZE];
/// Alias for Result<T, MachineError>.
pub type MachineResult<T> = Result<T, MachineError>;
/// Manages and maintains phase entries created de
/// uma mechanismo de math.
#[derive(Default, Debug)]
pub struct MachineCache {
    /// Actual cache entries of `Phase` objects.
    entries: BTreeSet<Rc<Phase>>,
    /// Tracks usage count per entry N of the
    /// cache.
    usages:  HashMap<MMInt, MMSize>,
}
/// Error occurred during the manipulation,
/// retrieval from/updating into a cache, or
/// directly in, a `Phase`.
#[derive(Debug)]
pub enum MachineError {
    /// Phase could not be found in a cache or
    /// other collection.
    PhaseNotFound,
}

/// A type can act as a cache for some other data
/// type.
pub trait Caches<K: Hash + ?Sized, V: Sized> {
    /// Remove an entry at the key from the cache
    /// returning the value, if it exists.
    fn drop(&mut self, key: &K) -> MachineResult<V>;
    /// Remove all invalid entries from the cache.
    /// Returns the dropped entries. The predicate
    /// determines if an entry is valid or not
    /// where:
    /// 
    /// let (valid, invalid) == (true, false);
    fn drop_invalid(&mut self, pred: impl FnMut(&Rc<V>) -> bool) -> MachineResult<Vec<V>>;
    /// Find a match in the cache for the given
    /// key.
    fn find(&mut self, key: &K) -> MachineResult<V>;
    /// Find the closest match in the cache for
    /// the given key.
    fn find_closest(&mut self, key: &K) -> MachineResult<V>;
    /// Find a match that meets the predicate
    /// searching in reverse order.
    fn find_rev(&mut self, key: &K, pred: impl FnMut(&&Rc<V>) -> bool) -> MachineResult<V>;
    /// Push a value to the cache at the given
    /// key.
    fn push(&mut self, entry: &V);
}

/// A type can return a new, empyt, instance of
/// itself.
pub trait Newable {
    /// Return a new blank instance.
    fn new() -> Self;
}

impl Newable for Phase {
    fn new() -> Phase {
        Default::default()
    }
}

impl Newable for MachineCache {
    fn new() -> MachineCache {
        MachineCache{
            entries: Default::default(),
            usages: Default::default(),
        }
    }
}

/// A type can manipulate an array as if it were
/// a `Phase`.
pub trait Phasable {
    /// Returns the `N` of the function call this
    /// phase represents.
    fn input(&self) -> &MMInt;
    /// The component values of the phase.
    fn phase(&self) -> [MMInt; PHASE_SIZE-PHASE_ARGC];
    /// Returns the result from the phase input.
    fn result(&self) -> &MMInt;
    /// Rotate phase elements to the right `K`
    /// places, preserving the `0th`and `1st`
    /// values in the phase.
    fn rotate(&mut self, k: MMSize);
}

impl Phasable for Phase {
    fn input(&self) -> &MMInt {
        self[0].borrow()
    }

    fn phase(&self) -> [MMInt; PHASE_SIZE-PHASE_ARGC] {
        self[PHASE_ARGC..].try_into().to_owned().expect("phase arguments")
    }

    fn result(&self) -> &MMInt {
        self[1].borrow()
    }

    fn rotate(&mut self, k: MMSize) {
        // Nothing to rotate if phase length is
        // too smol.
        if self.len()-PHASE_ARGC <= 1 { return; }
        self[PHASE_ARGC..].rotate_right(k);
    }
}

/// Type can do some calculation using the
/// `MathMachine` interface.
pub trait MathMachine {
    /// Performs the calculation this machine is
    /// supposed to do.
    fn calculate(&mut self, n: MMInt, phase: &mut Phase) -> MachineResult<Phase>;
}

impl MachineCache {
    /// Number of entries in this cache.
    pub fn len(&self) -> MMSize {
        self.entries.len()
    }

    /// Return the greatest count of iterations
    /// since last visit/use of any value in this
    /// cache.
    pub fn highest_usage(&self) -> MMSize {
        let mut us: Vec<&MMSize> = self.usages.values().collect();
        us.sort();
        **us.last().unwrap_or(&&0)
    }

    /// Update the usage of individual entry
    /// usages.
    fn update_usage(&mut self, filt: impl FnMut(&(&MMInt, &MMSize)) -> bool) {
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
    fn valid_usage(&self, key: &MMInt) -> bool {
        let key_usage = self.usages.get(key).expect("usage count");
        let gts_usage = &self.highest_usage();
        key_usage < gts_usage
    }
}

// Going off pattern by implementing Caches traits
// here to hopefully better illustrate usage
// specifically for a MathMachine `MachineCache`.
impl Caches<MMInt, Phase> for MachineCache {
    fn drop(&mut self, key: &MMInt) -> MachineResult<Phase> {
        match self.find(key) {
            Ok(phase) => {
                self.entries.remove(&phase);
                self.usages.remove(phase.input());
                Ok(phase.clone())
            },
            Err(err) => Err(err)
        }
    }

    fn drop_invalid(&mut self, mut pred: impl FnMut(&Rc<Phase>) -> bool) -> MachineResult<Vec<Phase>> {
        let mut retn = vec![];
        let entries_clone = self.entries.clone();
        let mut iter      = entries_clone.iter().rev();

        while let Some(p) = iter.next() {
            if pred(p) && self.valid_usage(p.input()) {
                continue;
            }
            retn.push(*p.to_owned());
            self.entries.remove(p);
            self.usages.remove(p.input());
        }
        Ok(retn)
    }

    fn find(&mut self, key: &MMInt) -> MachineResult<Phase> { 
        self.find_rev(key, |ph| ph.input() == key)
    }

    fn find_closest(&mut self, key: &MMInt) -> MachineResult<Phase> {
        // Find the closest-- would be--
        // preceeding cached phase.
        self.find_rev(key, |ph| ph.input() <= key)
    }

    fn find_rev(&mut self, key: &MMInt, pred: impl FnMut(&&Rc<Phase>) -> bool) -> MachineResult<Phase> {
        let entries_clone = self.entries.clone();
        let mut iter      = entries_clone.iter().rev();

        match iter.find(pred) {
            Some(phase) => {
                self.update_usage(|_| true);
                self.usages.insert(*key, 0);
                Ok(*phase.clone())
            },
            None => Err(MachineError::PhaseNotFound)
        }
    }

    fn push(&mut self, entry: &Phase) {
        let entry_rc = Rc::new(entry.to_owned());
        self.usages.insert(entry_rc.input().to_owned(), 0);
        self.entries.insert(entry_rc.clone());

        // Filter out entry inputs whose usage
        // count is 0; 
        self.update_usage(|(input, _)| **input != *entry_rc.input());
    }
}
