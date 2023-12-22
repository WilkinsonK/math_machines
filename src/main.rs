mod math_machine;

use std::cmp;
use math_machine::{Caches, MachineCache, MachineError, MMSize, Newable, Phase, Phasable, MMInt};
use rand;

/// Implements the Fibonacci sequence to calculate the
/// Nth value. Results are cached, with lookup in
/// reverse order, to find the closest value
/// calculated to a new N, if N does not already
/// exist.
#[derive(Default)]
struct FibonacciMachine {
    cache: MachineCache,
}

impl FibonacciMachine {

    /// Calculate the Nth value of the Fibonacci
    /// sequence.
    pub fn fibonacci(&mut self, n: MMInt) -> Result<MMInt, MachineError> {
        let mut phase;
        if let Ok(p) = self.cache.find_closest(&n) {
            phase = p.clone();
        } else {
            phase = Phase::new();
        }

        for _ in *phase.input()..n {
            phase.rotate(1);
            phase[1] = cmp::max(1, phase[2] + phase[3]);
        }
        phase[0] = n;

        self.cache.push(&phase.clone());
        Ok(phase.result().to_owned())
    }
}

fn main() {
    let machine = &mut FibonacciMachine::default();
    for _ in 0..50 {
        let n = rand::random::<MMInt>() % 50;
        let r = machine.fibonacci(n).expect("Nth value of Fibonacci");
        println!("fibonacci({n:-2}): {:-10}", r);
    }
}
