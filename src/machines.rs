use crate::math_machine::{Caches, MachineCache, MachineResult, MMSize, MathMachine, Newable, Phase, Phasable, MMInt};

use std::cmp;

/// Implements the Fibonacci sequence to calculate
/// the Nth value. Results are cached, with lookup
/// in reverse order, to find the closest value
/// calculated to a new N, if N does not already
/// exist.
#[derive(Default, Debug)]
pub struct FibonacciMachine {
    pub cache: MachineCache,
    pub max_entry_cap: MMSize,
    pub max_usage_age: MMSize,
}

/// Implements the sequence of prime numbers to
/// calculate the Nth value in the sequence.
/// Results are cached, with lookup
/// in reverse order, to find the closest value
/// calculated to a new N, if N does not already
/// exist.
#[derive(Default, Debug)]
pub struct PrimesMachine {
    pub cache: MachineCache,
    pub max_entry_cap: MMSize,
    pub max_usage_age: MMSize,
}

pub trait LRUCachable {
    /// Perform the cache entry drop algorithim.
    fn drop_invalid(&mut self) -> MachineResult<Vec<Phase>>;
    /// Internal cache has too many entries.
    fn is_too_big(&mut self) -> bool;
    /// Internal cache has entries that are
    /// greater than or equal to the maximum
    /// usage age.
    fn is_too_old(&mut self) -> bool;
    /// Get the maximum age an entry in the cache
    /// can reach before it becomes invalid.
    fn max_usage_age(&mut self) -> MMSize;
    /// Get the entry capacity for the internal
    /// cache.
    fn max_entry_cap(&mut self) -> MMSize;
}

/// Do the calculation of a math machine using
/// cache values to do lookups and cleanup using
/// an LRU scheme.
pub fn lru_calculate<MM: MathMachine + LRUCachable>(mm: &mut MM, n: MMInt) -> MachineResult<MMInt> {
    let mut phase = lru_find_phase(mm, n);
    lru_drop_if_capacity_met(mm);
    lru_do_calculation(mm, n, &mut phase)
}

fn lru_do_calculation<MM: MathMachine + LRUCachable>(mm: &mut MM, n: MMInt, phase: &mut Phase) -> MachineResult<MMInt> {
    match mm.calculate(n, phase) {
        Ok(phase) => {
            mm.update(&phase);
            Ok(phase.result().to_owned())
        },
        Err(m) => Err(m)
    }
}

fn lru_drop_if_capacity_met<MM: MathMachine + LRUCachable>(mm: &mut MM) {
    if mm.is_too_big() || mm.is_too_old() {
        let _ = mm.drop_invalid();
    }
}

fn lru_find_phase<MM: MathMachine + LRUCachable>(mm: &mut MM, n: MMInt) -> Phase {
    if let Ok(p) = mm.lookup(&n) {
        p.to_owned()
    } else {
        Phase::new()
    }
}

impl MathMachine for FibonacciMachine {
    fn calculate(&mut self, n: MMInt, phase: &mut Phase) -> MachineResult<Phase> {
        let (start, stahp) = (*phase.input(), n);
        phase[0] = n;
        for _ in start..stahp {
            phase.rotate(1);
            phase[1] = cmp::max(1, phase[2] + phase[3]);
        }
        Ok(phase.to_owned())
    }

    fn lookup(&mut self, n: &MMInt) -> MachineResult<Phase> {
        self.cache.find_closest(n)
    }

    fn update(&mut self, phase: &Phase) {
        self.cache.push(&phase.clone())
    }
}

impl LRUCachable for FibonacciMachine {
    fn drop_invalid(&mut self) -> MachineResult<Vec<Phase>> {
        self.cache.drop_invalid(|_| true)
    }

    fn is_too_big(&mut self) -> bool {
        self.max_entry_cap() >= self.cache.len()
    }

    fn is_too_old(&mut self) -> bool {
        self.max_usage_age() >= self.cache.highest_usage()
    }

    fn max_entry_cap(&mut self) -> MMSize {
        self.max_entry_cap
    }

    fn max_usage_age(&mut self) -> MMSize {
        self.max_usage_age
    }
}

impl PrimesMachine {
    /// Integer is a prime number or not.
    fn is_prime(&self, n: MMInt) -> bool {
        if n <= 1 { return false; }
        if n <= 3 { return true; }
        if n % 2 == 0 || n % 3 == 0 { return false; }

        let mut stepper: MMInt = 5;
        while stepper.pow(2) <= n {
            if n % stepper == 0 || n % (stepper + 2) == 0 {
                return false;
            }
            stepper += 6;
        }
        true
    }

    /// Get the next sequential prime number.
    fn next_prime(&self, mut n: MMInt) -> MMInt {
        if n == 0 { return 2; }
        if n == 1 || n == 2 { return n + 1; }
        n += 2;
        while !self.is_prime(n) {
            n += 2;
        }
        n
    }
}

impl MathMachine for PrimesMachine {
    fn calculate(&mut self, n: MMInt, phase: &mut Phase) -> MachineResult<Phase> {
        let (start, stahp) = (*phase.input(), n);
        phase[0] = n;
        for _ in start..stahp {
            phase[1] = self.next_prime(phase[1]);
        }
        Ok(phase.to_owned())
    }

    fn lookup(&mut self, n: &MMInt) -> MachineResult<Phase> {
        self.cache.find_closest(n)
    }

    fn update(&mut self, phase: &Phase) {
        self.cache.push(&phase.clone())
    }
}

impl LRUCachable for PrimesMachine {
    fn drop_invalid(&mut self) -> MachineResult<Vec<Phase>> {
        self.cache.drop_invalid(|_| true)
    }

    fn is_too_big(&mut self) -> bool {
        self.max_entry_cap() >= self.cache.len()
    }

    fn is_too_old(&mut self) -> bool {
        self.max_usage_age() >= self.cache.highest_usage()
    }

    fn max_entry_cap(&mut self) -> MMSize {
        self.max_entry_cap
    }

    fn max_usage_age(&mut self) -> MMSize {
        self.max_usage_age
    }
}
