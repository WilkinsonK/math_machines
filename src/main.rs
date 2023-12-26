pub mod math_machine;
pub mod machines;

use machines::{PrimesMachine, lru_calculate};
use math_machine::MMInt;

use rand;

fn main() {
    let machine = &mut PrimesMachine{
        cache: Default::default(),
        // TODO: integrate into cache machine.
        max_usage_age: 50,
        max_entry_cap: 128,
    };

    for _ in 0..50 {
        let n = rand::random::<MMInt>() % 50;
        let r = lru_calculate(machine, n).expect("Nth value of Primes");
        println!("prime({n:-2}): {:-3}", r);
    }
}
