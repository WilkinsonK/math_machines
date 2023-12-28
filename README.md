# Math Machines #

![GitHub License](https://img.shields.io/github/license/WilkinsonK/math_machines)
![GitHub Workflow Status (with event)](https://img.shields.io/github/actions/workflow/status/WilkinsonK/math_machines/rust.yml)

Math machines is a small collection of mathematical sequences expressed in the
form of methods to calculate the **Nth** number of a sequence.

```rust
use math_machines::{MMInt, Machine, lru_calculate, Fibonacci};
use rand;

let mut machine = Machine::new(Fibonacci{}, 128, 50);

for _ in 0..50 {
    let n = rand::random::<MMInt>() % 50;
    let r = lru_calculate(&mut machine, n).expect("Nth value of Fibonacci");
    println!("fibonacci({n:02}): {:-10}", r);
}
```

### sequences currently supported ###
- Fibonacci sequence
- Primes sequence

Machines that are defined in this project caches results at runtime using an
implementation of **LRU** (least recently used) where, once a machine's internal
cache has reached capacity, or the greatest age since usage has reach it's
maximum, cached entries are dropped.
