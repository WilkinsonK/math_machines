mod math_machine;

use std::cmp;
use std::collections::BTreeMap;
use rand;

type FibInt = u128;
type FibSize = u64;
type FibN = FibInt;
type FibPhase = [FibInt; 3];
type CacheEntry = (FibN, FibPhase);

#[derive(Debug)]
pub enum FibonacciError {
    PhaseNotFound,
    PhaseUpdateError,
}

pub trait FibBuilder<T> {
    fn new() -> T;
}

impl FibBuilder<FibPhase> for FibPhase {
    fn new() -> FibPhase {
        [0; 3]
    }
}

impl FibBuilder<CacheEntry> for CacheEntry {
    fn new() -> CacheEntry {
        (0, FibPhase::new())
    }
}

/// Implements the Fibonacci sequence to calculate the
/// Nth value. Results are cached, with lookup in
/// reverse order, to find the closest value
/// calculated to a new N, if N does not already
/// exist.
struct FibonacciMachine {
    entries: FibSize,
    cache: BTreeMap<FibInt, CacheEntry>,
}

impl FibBuilder<FibonacciMachine> for FibonacciMachine {
    fn new() -> FibonacciMachine {
        FibonacciMachine{entries: 0, cache: BTreeMap::new()}
    }
}

impl FibonacciMachine {

    /// Calculate the Nth value of the Fibonacci
    /// sequence.
    pub fn fibonacci(&mut self, n: FibN) -> Result<FibInt, FibonacciError> {
        let (last, mut phase) = match self.get(n) {
            Some(cached) => cached,
            None => return Err(FibonacciError::PhaseNotFound),
        };

        for _ in last..n {
            phase.rotate_left(1);
            phase[2] = cmp::max(1, phase[0] + phase[1]);
        }
    
        match self.put((n, phase)) {
            Some(v) => Ok(v),
            None => Err(FibonacciError::PhaseUpdateError)
        }
    }

    /// Get the closest phase previously cached in
    /// this machine.
    pub fn get(&mut self, n: FibN) -> Option<CacheEntry> {
        if let Some(cached) = self.get_last() {
            if cached.0 == n {
                return self.get_last()
            }
        }

        let default = CacheEntry::new();
        let it = self.cache
            .iter()
            .rev()
            .find(|c| c.0 <= &n);
        if it.is_some() {
            Some(it.unwrap_or_else(|| (&n, &default)).1.clone())
        } else {
            Some(default)
        }
    }

    /// Get the last entry cached in this machine.
    fn get_last(&mut self) -> Option<CacheEntry> {
        let entry = self.cache.last_entry();
        if let Some(entry) = entry {
            Some(entry.get().clone())
        } else {
            Some(CacheEntry::new())
        }
    }

    /// Push a new entry into the phase cache of
    /// this machine. Return the phase sum from
    /// the push.
    pub fn put(&mut self, entry: CacheEntry) -> Option<FibInt> {
        self.put_last(entry.clone());
        Some(entry.1[2])
    }

    /// Insert a new entry into the phase cache of
    /// this machine, updating the entry count.
    fn put_last(&mut self, entry: CacheEntry) {
        self.cache.insert(entry.0, entry);
        self.entries = self
            .cache
            .len() as FibSize;
    }
}

fn main() {
    let cache = &mut FibonacciMachine::new();
    for _ in 0..50 {
        let n = rand::random::<FibInt>() % 50;
        let r = cache.fibonacci(n).expect("Nth value of Fibonacci");
        println!("fibonacci({n:-2}): {:-10}", r);
    }
}
