use crate::phases::{MMFlt, MMInt, Phase};
use crate::machines::MachineResult;

use std::cmp;

/// Type can do some calculation using the
/// `Calculator` interface.
pub trait Calculator<T, I> {
    type Calculated;
    /// Performs the calculation this machine is
    /// supposed to do.
    fn calculate(&self, n: I, phase: &mut Self::Calculated) -> MachineResult<Self::Calculated>;
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

/// Implements the Harmonic series to calculate
/// the Nth value. Results are cached, with lookup
/// in reverse order, to find the closest value
/// calculated to a new N, if N does not already
/// exist.
#[derive(Debug)]
pub struct Harmonic;

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

impl Calculator<MMFlt, MMInt> for Harmonic {
    type Calculated = Phase<MMFlt, MMInt>;
    fn calculate(&self, n: MMInt, phase: &mut Self::Calculated) -> MachineResult<Self::Calculated> {
        let (start, stahp) = (&mut phase.input().to_owned(), n);
        phase.setinput(n);
        for _ in *start..stahp {
            phase[1] = phase[1]+1.0;
            phase[0] = phase[0] + 1.0/(phase[1].0);
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
