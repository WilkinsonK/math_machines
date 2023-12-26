pub mod caches;
pub mod machines;

use machines::{Machine, PrimesMachine, lru_calculate};
use caches::MMInt;
use rand;

fn main() {
    let machine = &mut Machine::new(PrimesMachine{}, 128, 50);

    for _ in 0..50 {
        let n = rand::random::<MMInt>() % 50;
        let r = lru_calculate(machine, n).expect("Nth value of Primes");
        println!("prime({n:-2}): {:-10}", r);
    }
}
