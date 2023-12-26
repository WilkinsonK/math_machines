# Math Machines #

Math machines is a small collection of mathematical sequences expressed in the
form of methods to calculate the **Nth** number of a sequence.

### sequences currently supported ###
- Fibonacci sequence
- Primes sequence

Machines that are defined in this project caches results at runtime using an
implementation of **LRU** (least recently used) where, once a machine's internal
cache has reached capacity, or the greatest age since usage has reach it's
maximum, cached entries are dropped.
