use crate::caches::{Caches, LRUCachable, MachineCache, MachineResult, MMSize, MathMachine, Newable, Phase, Phasable, MMInt};

use std::cmp;

pub struct Machine<MM: MathMachine> {
    cache: MachineCache,
    machine: MM,
    max_entry_cap: MMSize,
    max_usage_age: MMSize,
}

impl<MM: MathMachine> Machine<MM> {
    /// Do the internal calculation.
    fn calculate(&mut self, n: MMInt, phase: &mut Phase) -> MachineResult<Phase> {
        self.machine.calculate(n, phase)
    }
    /// Create a new instance of `Machine`.
    pub fn new(machine: MM, max_entries: MMSize, max_age: MMSize) -> Self {
        Machine{
            cache: MachineCache::default(),
            machine: machine,
            max_entry_cap: max_entries,
            max_usage_age: max_age
        }
    }
}

impl<MM: MathMachine> LRUCachable for Machine<MM> {
    fn drop_invalid(&mut self) -> MachineResult<Vec<Phase>> {
        self.cache.drop_invalid(|_| true)
    }
    fn is_too_big(&mut self) -> bool {
        self.max_entry_cap() >= self.cache.len()
    }
    fn is_too_old(&mut self) -> bool {
        self.max_usage_age() >= self.cache.highest_usage()
    }
    fn lookup(&mut self, n: &MMInt) -> MachineResult<Phase> {
        self.cache.find_closest(n)
    }
    fn max_entry_cap(&mut self) -> MMSize {
        self.max_entry_cap
    }
    fn max_usage_age(&mut self) -> MMSize {
        self.max_usage_age
    }
    fn update(&mut self, phase: &Phase) {
        self.cache.push(&phase.clone())
    }
}

/// Implements the Fibonacci sequence to calculate
/// the Nth value. Results are cached, with lookup
/// in reverse order, to find the closest value
/// calculated to a new N, if N does not already
/// exist.
///
/// ```
/// use math_machines::{Machine, FibonacciMachine, lru_calculate};
///
/// let machine = &mut Machine::new(FibonacciMachine{}, 128, 50);
/// let result  = lru_calculate(machine, 26).expect("26th fibonacci");
/// assert_eq!(result, 121393);
/// ```
pub struct FibonacciMachine;

/// Implements the sequence of prime numbers to
/// calculate the Nth value in the sequence.
/// Results are cached, with lookup
/// in reverse order, to find the closest value
/// calculated to a new N, if N does not already
/// exist.
///
/// ```
/// use math_machines::{Machine, PrimesMachine, lru_calculate};
///
/// let machine = &mut Machine::new(PrimesMachine{}, 128, 50);
/// let result  = lru_calculate(machine, 26).expect("26th prime");
/// assert_eq!(result, 101);
/// ```
pub struct PrimesMachine;

/// Do the calculation of a math machine using
/// cache values to do lookups and cleanup using
/// an LRU scheme.
pub fn lru_calculate<MM: MathMachine>(mm: &mut Machine<MM>, n: MMInt) -> MachineResult<MMInt> {
    let mut phase = lru_find_phase(mm, n);
    lru_drop_if_capacity_met(mm);
    lru_do_calculation(mm, n, &mut phase)
}

/// Perform a raw calculation for the Nth value of
/// a math machine. This function executes without
/// doing any caching operations.
pub fn raw_calculate<MM: MathMachine>(mm: &mut Machine<MM>, n: MMInt) -> MachineResult<MMInt> {
    match mm.calculate(n, &mut Phase::new()) {
        Ok(phase) => Ok(phase.result().to_owned()),
        Err(m) => Err(m)
    }
}

fn lru_do_calculation<MM: MathMachine>(mm: &mut Machine<MM>, n: MMInt, phase: &mut Phase) -> MachineResult<MMInt> {
    match mm.calculate(n, phase) {
        Ok(phase) => {
            mm.update(&phase);
            Ok(phase.result().to_owned())
        },
        Err(m) => Err(m)
    }
}

fn lru_drop_if_capacity_met<LC: LRUCachable>(mm: &mut LC) {
    if mm.is_too_big() || mm.is_too_old() {
        let _ = mm.drop_invalid();
    }
}

fn lru_find_phase<LC: LRUCachable>(mm: &mut LC, n: MMInt) -> Phase {
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
}

impl PrimesMachine {
    /// Integer is a prime number or not.
    ///
    /// ```
    /// use math_machines::PrimesMachine;
    ///
    /// assert_eq!(PrimesMachine::is_prime(98), false);
    /// assert_eq!(PrimesMachine::is_prime(144), false);
    /// assert_eq!(PrimesMachine::is_prime(181), true);
    /// ```
    pub fn is_prime(n: MMInt) -> bool {
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
    ///
    /// ```
    /// use math_machines::PrimesMachine;
    ///
    /// assert_eq!(PrimesMachine::next_prime(3517), 3527);
    /// assert_eq!(PrimesMachine::next_prime(7489), 7499);
    /// ```
    pub fn next_prime(mut n: MMInt) -> MMInt {
        if n == 0 { return 2; }
        if n == 1 || n == 2 { return n + 1; }
        n += 2;
        while !PrimesMachine::is_prime(n) {
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
            phase[1] = PrimesMachine::next_prime(phase[1]);
        }
        Ok(phase.to_owned())
    }
}
