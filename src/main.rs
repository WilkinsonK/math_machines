use math_machines::{MMInt, Machine, lru_calculate, Fibonacci};
use rand;

fn main() {
    let mut machine = Machine::new(Fibonacci{}, 128, 50);

    for _ in 0..50 {
        let n = rand::random::<MMInt>() % 50;
        let r = lru_calculate(&mut machine, n).expect("Nth value of Fibonacci");
        println!("fibonacci({n:02}): {:-10}", r);
    }
}
