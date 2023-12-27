mod caches;
mod machines;
mod phases;

pub use phases::{MMFlt, MMInt};
pub use machines::{FibonacciMachine, PrimesMachine, Machine, lru_calculate, raw_calculate};
