mod math_machine;

use std::cmp;
use math_machine::{Caches, MachineCache, MachineError, MMSize, Newable, Phase, Phasable, MMInt};
use rand;

/// Implements the Fibonacci sequence to calculate the
/// Nth value. Results are cached, with lookup in
/// reverse order, to find the closest value
/// calculated to a new N, if N does not already
/// exist.
#[derive(Default, Debug)]
struct FibonacciMachine {
    cache: MachineCache,
    max_entry_cap: MMSize,
    max_usage_age: MMSize,
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

        let too_old = self.cache.highest_usage() >= self.max_usage_age;
        let too_big = self.cache.len() >= self.max_entry_cap;
        if too_old || too_big {
            let _ = self.cache
                .drop_invalid(|_| true)
                .expect("dropped entries");
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
    let machine = &mut FibonacciMachine{
        cache: Default::default(),
        // TODO: integrate into cache machine.
        max_usage_age: 50,
        max_entry_cap: 128,
    };

    for _ in 0..50 {
        let n = rand::random::<MMInt>() % 50;
        let r = machine.fibonacci(n).expect("Nth value of Fibonacci");
        println!("fibonacci({n:-2}): {:-10}", r);
    }
}
