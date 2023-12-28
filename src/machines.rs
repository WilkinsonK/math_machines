use crate::caches::{Caches, MachineCache, CacheResult};
use crate::phases::{MMInt, MMSize, Newable, Phase};

use std::borrow::BorrowMut;
use std::cmp;
use std::fmt::Debug;
use std::hash::Hash;

/// Error occurred during some calculation.
#[derive(Debug)]
pub enum MachineError {}
/// Alias for Result<T, MachineError>.
type MachineResult<T> = Result<T, MachineError>;

/// Type can do some calculation using the
/// `Calculator` interface.
pub trait Calculator<T, I> {
    type Calculated;
    /// Performs the calculation this machine is
    /// supposed to do.
    fn calculate(&self, n: I, phase: &mut Self::Calculated) -> MachineResult<Self::Calculated>;
}

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

/// Implements the Fibonacci sequence to calculate
/// the Nth value. Results are cached, with lookup
/// in reverse order, to find the closest value
/// calculated to a new N, if N does not already
/// exist.
///
/// ```
/// use math_machines::{Machine, Fibonacci, lru_calculate};
///
/// let machine = &mut Machine::new(Fibonacci{}, 128, 50);
/// let result  = lru_calculate(machine, 26).expect("26th fibonacci");
/// assert_eq!(result, 121393);
/// ```
#[derive(Debug)]
pub struct Fibonacci;

/// Implements the sequence of prime numbers to
/// calculate the Nth value in the sequence.
/// Results are cached, with lookup
/// in reverse order, to find the closest value
/// calculated to a new N, if N does not already
/// exist.
///
/// ```
/// use math_machines::{Machine, Primes, lru_calculate};
///
/// let machine = &mut Machine::new(Primes{}, 128, 50);
/// let result  = lru_calculate(machine, 26).expect("26th prime");
/// assert_eq!(result, 101);
/// ```
#[derive(Debug)]
pub struct Primes;

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

impl Calculator<MMInt, MMInt> for Fibonacci {
    type Calculated = Phase<MMInt, MMInt>;
    fn calculate(&self, n: MMInt, phase: &mut Self::Calculated) -> MachineResult<Self::Calculated> {
        let (start, stahp) = (&mut phase.input().to_owned(), n);
        phase.setinput(n);
        for _ in *start..stahp {
            phase.rotate(1);
            phase[0] = cmp::max(1, phase[1] + phase[2]);
        }
        Ok(phase.to_owned())
    }
}

impl Primes {
    /// Integer is a prime number or not.
    ///
    /// ```
    /// use math_machines::Primes;
    ///
    /// assert_eq!(Primes::is_prime(98), false);
    /// assert_eq!(Primes::is_prime(144), false);
    /// assert_eq!(Primes::is_prime(181), true);
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
    /// use math_machines::Primes;
    ///
    /// assert_eq!(Primes::next_prime(3517), 3527);
    /// assert_eq!(Primes::next_prime(7489), 7499);
    /// ```
    pub fn next_prime(mut n: MMInt) -> MMInt {
        if n == 0 { return 2; }
        if n == 1 || n == 2 { return n + 1; }
        n += 2;
        while !Primes::is_prime(n) {
            n += 2;
        }
        n
    }
}

impl Calculator<MMInt, MMInt> for Primes {
    type Calculated = Phase<MMInt, MMInt>;
    fn calculate(&self, n: MMInt, phase: &mut Self::Calculated) -> MachineResult<Self::Calculated> {
        let (start, stahp) = (phase.input().to_owned(), n);
        phase.setinput(n);
        for _ in start..stahp {
            phase[0] = Primes::next_prime(phase[0]);
        }
        Ok(phase.to_owned())
    }
}
