use math_machines::{MMInt, Machine, lru_calculate, Harmonic};
use rand;

fn main() {
    let mut machine = Machine::new(Harmonic{}, 128, 50);
    for _ in 0..50 {
        let n = rand::random::<MMInt>() % 50;
        let r = lru_calculate(&mut machine, n).expect("Nth value of Harmonic");
        println!("harmonic({n:02}): {:-8.5}", r.0);
    }
}
