use std::fmt::{Debug, Display};
use std::ops::{Index, IndexMut};
use std::slice::SliceIndex;

const PHASE_SIZE: MMSize = 3;

/// Type can create a new instance of itself.
pub trait Newable {
    fn new() -> Self;
}
/// Variable type alias for the size of integer
/// math machines use.
pub type MMInt = u128;
/// Variable type alias for the size of float
/// math machines use.
pub type MMFlt = f64;
/// Variable type alias for the `size` type
/// math machines use.
pub type MMSize = usize;
/// Actual, or internal, phase slice of a `Phase`
/// instance.
type PhaseActual<T> = [T; PHASE_SIZE];
/// A slice of values used to calculate some
/// result. Phases are processed by caches to make
/// calculation of large numbers faster. 0th and
/// 1st values are reserved for the result of the
/// phase where remaining `MMNumeric`s
/// `(2nd, 3rd, 4th, ...)` are the arguments to
/// achieve said result.
#[derive(Clone, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct Phase<T, I> {
    phase:  PhaseActual<T>,
    input:  I,
}

impl<T: Default, I: Default> Phase<T, I> {
    /// Returns the `N` of the function call this
    /// phase represents.
    pub fn input(&self) -> &I {
        &self.input
    }
    /// The component values of the phase.
    pub fn phase(&self) -> &PhaseActual<T> {
        &self.phase
    }
    /// Returns the result from the phase input.
    pub fn result(&self) -> &T {
        &self.phase[0]
    }
    /// Rotate phase elements to the right `K`
    /// places, preserving the `0th`and `1st`
    /// values in the phase.
    pub fn rotate(&mut self, k: MMSize) {
        self.phase.rotate_right(k)
    }
    /// Set the input of this phase.
    pub fn setinput(&mut self, n: &I)
    where
        I: Copy
    {
        self.input = n.to_owned();
    }
}

impl<T, I> Debug for Phase<T, I>
where
    T: Debug,
    I: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({:?}: {:?})", self.input, self.phase)
    }
}

impl<T, I> Display for Phase<T, I>
where
    T: Debug,
    I: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}{:?})", self.input, self.phase)
    }
}

impl<Idx, T, I> Index<Idx> for Phase<T, I>
where
    Idx: SliceIndex<[T]>,
{
    type Output = Idx::Output;
    fn index(&self, index: Idx) -> &Self::Output {
        &self.phase[index]
    }
}

impl<Idx, T, I> IndexMut<Idx> for Phase<T, I>
where
    Idx: SliceIndex<[T]>,
{
    fn index_mut(&mut self, index: Idx) -> &mut Self::Output {
        &mut self.phase[index]
    }
}

impl<T, I>Newable for Phase<T, I>
where
    T: Default,
    I: Default,
{
    /// Return a new instance of a `Phase`.
    fn new() -> Self {
        Self {phase: Default::default(), input: Default::default()}
    }
}
