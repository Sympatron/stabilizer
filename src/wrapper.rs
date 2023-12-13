use crate::{InitializedValue, Monotonic, State, TimedDebouncer};

/// Trait to interface with [`DebouncedInput`].
pub trait Input<T> {
    /// Read the current state of the input
    fn read(&mut self) -> T;
}

/// Generic debouncing wrapper for any input implementing [`Input`].
pub struct DebouncedInput<M: Monotonic, T: Copy, I> {
    debouncer: TimedDebouncer<M, T, InitializedValue<T>>,
    input: I,
}

impl<M, T, I> DebouncedInput<M, T, I>
where
    I: Input<T>,
    M: Monotonic,
    M::Duration: Copy,
    T: Copy + PartialEq,
{
    /// Creates a new [`DebouncedInput`] by wrapping an [`Input`]
    pub fn new(mut input: I, debounce_time: M::Duration) -> Self {
        Self {
            debouncer: TimedDebouncer::new(input.read(), debounce_time),
            input,
        }
    }
    /// Read the current state of the input.
    pub fn read(&mut self) -> State<T, InitializedValue<T>> {
        self.debouncer.update(self.input.read())
    }
}

impl<M, T, I> DebouncedInput<M, T, I>
where
    M: Monotonic,
    M::Duration: Copy,
    T: Copy + PartialEq,
{
    /// Read the last stable state of the input.
    pub fn read_stable(&self) -> T {
        self.debouncer.read_stable()
    }
}

impl<M, T, I> Input<T> for DebouncedInput<M, T, I>
where
    I: Input<T>,
    M: Monotonic,
    M::Duration: Copy,
    T: Copy + PartialEq,
{
    fn read(&mut self) -> T {
        self.read().stable()
    }
}

impl<I: ehal0::digital::v2::InputPin> Input<Result<ehal0::digital::v2::PinState, I::Error>> for I {
    fn read(&mut self) -> Result<ehal0::digital::v2::PinState, I::Error> {
        if self.is_high()? {
            Ok(ehal0::digital::v2::PinState::High)
        } else {
            Ok(ehal0::digital::v2::PinState::Low)
        }
    }
}

impl<I: ehal1::digital::InputPin> Input<Result<ehal1::digital::PinState, I::Error>> for I {
    fn read(&mut self) -> Result<ehal1::digital::PinState, I::Error> {
        if self.is_high()? {
            Ok(ehal1::digital::PinState::High)
        } else {
            Ok(ehal1::digital::PinState::Low)
        }
    }
}
