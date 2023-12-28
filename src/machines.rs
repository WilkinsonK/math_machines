use crate::caches::{Caches, MachineCache, CacheResult};
use crate::calculators::Calculator;
use crate::phases::{MMSize, Newable, Phase};

use std::fmt::Debug;
use std::hash::Hash;

/// Error occurred during some calculation.
#[derive(Debug)]
pub enum MachineError {}
/// Alias for Result<T, MachineError>.
pub type MachineResult<T> = Result<T, MachineError>;


/// Handles all the operations from `calculate`,
/// `update` and `lookup` on the cache, and
/// cleanup on the cache as LRU is needed.
#[derive(Debug)]
pub struct Machine<T, I, MM>
where
    T: Clone + Default + Ord,
    I: Clone + Default + Eq + Hash + Ord + PartialEq,
    MM: Calculator<T, I>,
{
    cache: MachineCache<T, I>,
    machine: MM,
    max_entry_cap: MMSize,
    max_usage_age: MMSize,
}

impl<T, I, MM> Machine<T, I, MM>
where
    T: Clone + Debug + Default + Ord,
    I: Clone + Copy + Debug + Default + Eq + Hash + Ord + PartialEq,
    MM: Calculator<T, I>,
{
    /// Do the internal calculation.
    fn calculate(&self, n: I, phase: &mut MM::Calculated) -> MachineResult<MM::Calculated> {
        self.machine.calculate(n, phase)
    }
    /// Create a new instance of `Machine`.
    pub fn new(machine: MM, max_entries: MMSize, max_age: MMSize) -> Self {
        Machine{
            cache: MachineCache::new(),
            machine: machine,
            max_entry_cap: max_entries,
            max_usage_age: max_age
        }
    }
    fn drop_invalid(&mut self) -> CacheResult<Vec<Phase<T, I>>> {
        self.cache.drop_invalid(|_| true)
    }
    fn is_too_big(&self) -> bool {
        self.cache.len() >= self.max_entry_cap()
    }
    fn is_too_old(&self) -> bool {
        self.cache.highest_usage() >= self.max_usage_age()
    }
    fn lookup(&mut self, n: I) -> CacheResult<Phase<T, I>> {
        self.cache.find_closest(n)
    }
    fn max_entry_cap(&self) -> MMSize {
        self.max_entry_cap
    }
    fn max_usage_age(&self) -> MMSize {
        self.max_usage_age
    }
    fn update(&mut self, phase: Phase<T, I>) {
        let ret = self.cache.push(phase.clone());
        ret
    }
}

/// Do the calculation of a math machine using
/// cache values to do lookups and cleanup using
/// an LRU scheme.
pub fn lru_calculate<T, I, MM>(mm: &mut Machine<T, I, MM>, n: I) -> MachineResult<T>
where
    T: Clone + Debug + Default + Ord,
    I: Clone + Debug + Copy + Default + Eq + Hash + Ord + PartialEq,
    MM: Calculator<T, I, Calculated = Phase<T, I>>
{
    let mut phase = lru_find_phase(mm, n.clone());
    lru_drop_if_capacity_met(mm);
    lru_do_calculation(mm, n, &mut phase)
}

/// Perform a raw calculation for the Nth value of
/// a math machine. This function executes without
/// doing any caching operations.
pub fn raw_calculate<T, I, MM>(mm: &Machine<T, I, MM>, n: I) -> MachineResult<T>
where
    T: Clone + Debug + Default + Ord,
    I: Clone + Copy + Debug + Default + Eq + Hash + Ord + PartialEq,
    MM: Calculator<T, I, Calculated = Phase<T, I>>,
{
    match mm.calculate(n, &mut MM::Calculated::new()) {
        Ok(calc) => Ok(calc.result().to_owned()),
        Err(m) => Err(m)
    }
}

fn lru_do_calculation<T, I, MM>(mm: &mut Machine<T, I, MM>, n: I, phase: &mut MM::Calculated) -> MachineResult<T>
where
    T: Clone + Debug + Default + Ord,
    I: Clone + Copy + Debug + Default + Eq + Hash + Ord + PartialEq,
    MM: Calculator<T, I, Calculated = Phase<T, I>>,
{
    match mm.calculate(n, phase) {
        Ok(calc) => {
            mm.update(calc.clone());
            Ok(calc.result().to_owned())
        },
        Err(m) => Err(m)
    }
}

fn lru_drop_if_capacity_met<T, I, MM>(mm: &mut Machine<T, I, MM>)
where
    T: Clone + Debug + Default + Ord,
    I: Clone + Copy + Debug + Default + Eq + Hash + Ord + PartialEq,
    MM: Calculator<T, I>,
{
    if mm.is_too_big() || mm.is_too_old() {
        let _ = mm.drop_invalid().expect("dropped values");
    }
}

fn lru_find_phase<T, I, MM>(mm: &mut Machine<T, I, MM>, n: I) -> Phase<T, I>
where
    T: Clone + Debug + Default + Ord,
    I: Clone + Copy + Debug + Default + Eq + Hash + Ord + PartialEq,
    MM: Calculator<T, I>,
{
    if let Ok(p) = mm.lookup(n) {
        p.to_owned()
    } else {
        Phase::new()
    }
}
