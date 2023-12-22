use std::collections::{BTreeSet, HashMap};
use std::borrow::Borrow;
use std::hash::Hash;
use std::rc::Rc;

const PHASE_SIZE: MMSize = 4;

/// Variable type alias for the size of integer
/// math machines use.
pub type MMInt = u128;
/// Variable type alias for the size of floats
/// math machines use.
pub type MMflt = f64;
/// Union of `MMInt` and `MMflt` data types.
#[derive(Clone, Copy, Default)]
pub struct MMNumeric {
    int: MMInt,
    flt: MMflt
}
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
pub type Phase = [MMNumeric; PHASE_SIZE];
/// Manages and maintains phase entries created de
/// uma mechanismo de math.
pub struct MachineCache {
    /// Actual cache entries of `Phase` objects.
    entries: BTreeSet<Rc<Phase>>,
    /// Tracks usage count per entry N of the
    /// cache.
    usages:  HashMap<MMNumeric, MMSize>,
}
/// Error occurred during the manipulation,
/// retrieval from/updating into a cache, or
/// directly in, a `Phase`.
pub enum MachineError {
    /// Phase could not be found in a cache or
    /// other collection.
    PhaseNotFound,
    /// The potential calculation is too large
    /// for MM numeric types.
    PhaseTooLarge,
}

/// A type can act as a cache for some other data
/// type.
pub trait Caches<K: Hash + ?Sized, V: Sized> {
    /// Remove an entry at the key from the cache
    /// returning the value, if it exists.
    fn drop(&mut self, key: &K) -> Result<V, MachineError>;
    /// Remove all invalid entries from the cache.
    /// Returns the dropped entries. The predicate
    /// determines if an entry is valid or not
    /// where:
    /// 
    /// let (valid, invalid) == [true, false];
    fn drop_invalid(&mut self, pred: impl FnMut(&Rc<V>) -> bool) -> Result<Vec<V>, MachineError>;
    /// Find a match in the cache for the given
    /// key.
    fn find(&mut self, key: &K) -> Result<V, MachineError>;
    /// Find the closest match in the cache for
    /// the given key.
    fn find_closest(&mut self, key: &K) -> Result<V, MachineError>;
    /// Find a match that meets the predicate
    /// searching in reverse order.
    fn find_rev(&mut self, key: &K, pred: impl FnMut(&&Rc<V>) -> bool) -> Result<V, MachineError>;
    /// Push a value to the cache at the given
    /// key.
    fn push(&mut self, entry: &V);
}

impl Eq for MMNumeric {}

impl Hash for MMNumeric {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.int.hash(state)
    }
}

pub trait Indexable {}
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

impl Ord for MMNumeric {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.int.cmp(&other.int)
    }
}

impl PartialEq for MMNumeric {
    fn eq(&self, other: &Self) -> bool {
        self.int.ne(&other.int)
    }
}

impl PartialOrd for MMNumeric {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.int.partial_cmp(&other.int)
    }
}

/// A type can manipulate an array as if it were
/// a `Phase`.
pub trait Phasable {
    /// Returns the `N` of the function call this
    /// phase represents.
    fn input(&self) -> &MMNumeric;
    /// Returns the result from the phase input.
    fn result(&self) -> &MMNumeric;
    /// Rotate phase elements to the right `K`
    /// places, preserving the `0th`and `1st`
    /// values in the phase.
    fn rotate(&mut self, k: MMSize);
}

impl Phasable for Phase {
    fn input(&self) -> &MMNumeric {
        self[1].borrow()
    }

    fn result(&self) -> &MMNumeric {
        self[0].borrow()
    }

    fn rotate(&mut self, k: MMSize) {
        // Nothing to rotate if phase length is
        // too smol.
        if self.len() <= 3 { return; }
        self[2..].rotate_right(k);
    }
}

impl MachineCache {
    /// Update the usage of individual entry
    /// usages.
    fn update_usage(&mut self, filt: impl FnMut(&(&MMNumeric, &MMSize)) -> bool) {
        let usage_clone = self.usages.clone();
        let mut iter = usage_clone.iter().filter(filt);
        // Iterate through the usages, after push
        // of new data, to increment usage by 1
        // for each usage record.
        while let Some((input, usage)) = iter.next() {
            self.usages.insert(*input, usage+1);
        }
    }

    fn valid_usage(&self, key: &MMNumeric) -> bool {
        let mut us: Vec<&MMSize> = self.usages.values().collect();
        us.sort();

        let key_usage = self.usages.get(key).expect("usage count");
        let gts_usage = us.last().expect("greatest usage");
        key_usage < gts_usage
    }
}

// Going off pattern by implementing Caches traits
// here to hopefully better illustrate usage
// specifically for a MathMachine `MachineCache`.
impl Caches<MMNumeric, Phase> for MachineCache {
    fn drop(&mut self, key: &MMNumeric) -> Result<Phase, MachineError> {
        match self.find(key) {
            Ok(phase) => {
                self.entries.remove(&phase);
                self.usages.remove(phase.input());
                Ok(phase.clone())
            },
            Err(err) => Err(err)
        }
    }

    fn drop_invalid(&mut self, mut pred: impl FnMut(&Rc<Phase>) -> bool) -> Result<Vec<Phase>, MachineError> {
        let mut retn = vec![];
        let mut iter = self.entries.iter().rev();


        while let Some(p) = iter.next() {
            if pred(p) && self.valid_usage(p.input()) {
                continue;
            }
            retn.push(*p.to_owned());
        }
        Ok(retn)
    }

    fn find(&mut self, key: &MMNumeric) -> Result<Phase, MachineError> { 
        self.find_rev(key, |ph| ph.input() == key)
    }

    fn find_closest(&mut self, key: &MMNumeric) -> Result<Phase, MachineError> {
        // Find the closest-- would be--
        // preceeding cached phase.
        self.find_rev(key, |ph| ph.input() <= key)
    }

    fn find_rev(&mut self, key: &MMNumeric, pred: impl FnMut(&&Rc<Phase>) -> bool) -> Result<Phase, MachineError> {
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
