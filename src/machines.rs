use crate::caches::{Caches, LRUCachable, MachineCache, CacheResult};
use crate::phases::{MMInt, MMSize, Newable, Phase};

use std::cmp;
use std::fmt::Debug;
use std::hash::Hash;

/// Error occurred during some calculation.
#[derive(Debug)]
pub enum MachineError {}
/// Alias for Result<T, MachineError>.
type MachineResult<T> = Result<T, MachineError>;

/// Type can do some calculation using the
/// `MathMachine` interface.
pub trait MathMachine<T, I> {
    type Calculated;
    /// Performs the calculation this machine is
    /// supposed to do.
    fn calculate(&mut self, n: I, phase: &mut Self::Calculated) -> MachineResult<Self::Calculated>;
}

#[derive(Debug)]
pub struct Machine<T, I, MM>
where
    T: Clone + Default + Ord,
    I: Clone + Default + Eq + Hash + Ord + PartialEq,
    MM: MathMachine<T, I>,
{
    cache: MachineCache<T, I>,
    machine: MM,
    max_entry_cap: MMSize,
    max_usage_age: MMSize,
}

impl<T, I, MM: MathMachine<T, I>> Machine<T, I, MM>
where
    T: Clone + Default + Ord,
    I: Clone + Default + Eq + Hash + Ord + PartialEq,
{
    /// Do the internal calculation.
    fn calculate(&mut self, n: I, phase: &mut MM::Calculated) -> MachineResult<MM::Calculated> {
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

impl<T, I, MM> LRUCachable<I> for Machine<T, I, MM>
where
    T: Clone + Debug + Default + Ord,
    I: Clone + Copy + Debug + Default + Eq + Hash + Ord + PartialEq,
    MM: MathMachine<T, I>,
{
    type Cached = Phase<T, I>;
    fn drop_invalid(&mut self) -> CacheResult<Vec<Self::Cached>> {
        self.cache.drop_invalid(|_| true)
    }
    fn is_too_big(&mut self) -> bool {
        self.max_entry_cap() >= self.cache.len()
    }
    fn is_too_old(&mut self) -> bool {
        self.max_usage_age() >= self.cache.highest_usage()
    }
    fn lookup(&mut self, n: &I) -> CacheResult<Self::Cached> {
        self.cache.find_closest(n)
    }
    fn max_entry_cap(&mut self) -> MMSize {
        self.max_entry_cap
    }
    fn max_usage_age(&mut self) -> MMSize {
        self.max_usage_age
    }
    fn update(&mut self, phase: &Self::Cached) {
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
#[derive(Debug)]
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
#[derive(Debug)]
pub struct PrimesMachine;

/// Do the calculation of a math machine using
/// cache values to do lookups and cleanup using
/// an LRU scheme.
pub fn lru_calculate<T, I, MM>(mm: &mut Machine<T, I, MM>, n: I) -> MachineResult<T>
where
    T: Clone + Debug + Default + Ord,
    I: Clone + Debug + Copy + Default + Eq + Hash + Ord + PartialEq,
    MM: MathMachine<T, I, Calculated = Phase<T, I>>
{
    let mut phase = lru_find_phase::<T, I, Machine<T, I, MM>>(mm, n.clone());
    lru_drop_if_capacity_met(mm);
    lru_do_calculation(mm, n, &mut phase)
}

/// Perform a raw calculation for the Nth value of
/// a math machine. This function executes without
/// doing any caching operations.
pub fn raw_calculate<T, I, MM>(mm: &mut Machine<T, I, MM>, n: I) -> MachineResult<T>
where
    T: Clone + Default + Ord,
    I: Clone + Default + Eq + Hash + Ord + PartialEq,
    MM: MathMachine<T, I, Calculated = Phase<T, I>>,
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
    MM: MathMachine<T, I, Calculated = Phase<T, I>>,
{
    match mm.calculate(n, phase) {
        Ok(calc) => {
            mm.update(&calc);
            Ok(calc.result().to_owned())
        },
        Err(m) => Err(m)
    }
}

fn lru_drop_if_capacity_met<I, LC>(mm: &mut LC)
where
    I: Clone + Default + Eq + Hash + Ord + PartialEq,
    LC: LRUCachable<I>,
{
    if mm.is_too_big() || mm.is_too_old() {
        let _ = mm.drop_invalid();
    }
}

fn lru_find_phase<T, I, LC>(mm: &mut LC, n: I) -> LC::Cached
where
    T: Default + Ord,
    I: Default + Eq + Hash + Ord + PartialEq,
    LC: LRUCachable<I>,
    LC::Cached: Clone + Newable,
{
    if let Ok(p) = mm.lookup(&n) {
        p.to_owned()
    } else {
        LC::Cached::new()
    }
}

impl MathMachine<MMInt, MMInt> for FibonacciMachine {
    type Calculated = Phase<MMInt, MMInt>;
    fn calculate(&mut self, n: MMInt, phase: &mut Self::Calculated) -> MachineResult<Self::Calculated> {
        let (start, stahp) = (&mut phase.input().to_owned(), n);
        phase.setinput(&n);
        for _ in *start..stahp {
            phase.rotate(1);
            phase[0] = cmp::max(1, phase[1] + phase[2]);
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

impl MathMachine<MMInt, MMInt> for PrimesMachine {
    type Calculated = Phase<MMInt, MMInt>;
    fn calculate(&mut self, n: MMInt, phase: &mut Self::Calculated) -> MachineResult<Self::Calculated> {
        let (start, stahp) = (phase.input().to_owned(), n);
        phase.setinput(&n);
        for _ in start..stahp {
            phase[0] = PrimesMachine::next_prime(phase[0]);
        }
        Ok(phase.to_owned())
    }
}
