use math_machines::{Machine, lru_calculate, PrimesMachine, FibonacciMachine};
// use rand;

fn main() {
    let machine = &mut Machine::new(FibonacciMachine{}, 128, 50);

    for n in 0..8 {
        // let n = rand::random::<MMInt>() % 50;
        let r = lru_calculate(machine, n).expect("Nth value of Fibonacci");
        println!("fibonacci({n:02}): {:-10}", r);
    }
    println!("{machine:?}")
}
