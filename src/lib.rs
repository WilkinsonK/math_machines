pub mod caches;
pub mod calculators;
pub mod machines;
pub mod phases;

pub use caches::{Caches, MachineCache};
pub use calculators::*;
pub use machines::{Machine, lru_calculate, raw_calculate};
pub use phases::{MMFlt, MMInt};

/// ```
/// use math_machines as mm;
/// let machine = &mut mm::Machine::new(mm::Fibonacci{}, 128, 50);
///
/// for n in 0..21 {
///     let r = mm::lru_calculate(machine, n).expect("Nth value of Fibonacci");
/// }
/// ```
#[cfg(doctest)]
struct MathMachines;
