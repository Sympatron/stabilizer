use core::{cell::RefCell, convert::Infallible};

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
#[cfg(feature = "ehal0")]
impl<M, I> InputPinV0 for DebouncedInput<M, Result<PinStateV0, Infallible>, I>
where
    I: InputPinV0,
    M: Monotonic,
    M::Duration: Copy,
{
    type Error = Infallible;
    fn is_high(&self) -> Result<bool, Self::Error> {
        self.read_stable().and_then(|s| Ok(s == PinStateV0::High))
    }
    fn is_low(&self) -> Result<bool, Self::Error> {
        self.read_stable().and_then(|s| Ok(s == PinStateV0::Low))
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "ehal1")))]
#[cfg(feature = "ehal1")]
impl<M: Monotonic, T: Copy, I> ehal1::digital::ErrorType
    for DebouncedInput<M, Result<T, Infallible>, I>
{
    type Error = Infallible;
}
#[cfg_attr(docsrs, doc(cfg(feature = "ehal1")))]
#[cfg(feature = "ehal1")]
impl<M: Monotonic, T: Copy, I> ehal1::digital::ErrorType
    for DebouncedInputRef<M, Result<T, Infallible>, I>
{
    type Error = Infallible;
}
struct DebouncedInputRef<M: Monotonic, T: Copy, I>(RefCell<DebouncedInput<M, T, I>>);

// #[cfg_attr(docsrs, doc(cfg(feature = "ehal0")))]
// #[cfg(feature = "ehal0")]
// impl<M, I> InputPinV0
//     for DebouncedInputRef<M, Result<PinStateV0, Infallible>, I>
// where
//     I: InputPinV1<Error = Infallible>,
//     M: Monotonic,
//     M::Duration: Copy,
// {
//     type Error = Infallible;
//     fn is_high(&self) -> Result<bool, Self::Error> {
//         let input = &mut *self.0.borrow_mut();
//         Ok(input.read().unwrap_safe().stable() == PinStateV1::High)
//     }
//     fn is_low(&self) -> Result<bool, Self::Error> {
//         let input = &mut *self.0.borrow_mut();
//         Ok(input.read().unwrap_safe().stable() == PinStateV1::Low)
//     }
// }
// #[cfg_attr(docsrs, doc(cfg(feature = "ehal1")))]
// #[cfg(feature = "ehal1")]
// impl<M, I> InputPinV1
//     for DebouncedInputRef<M, Result<PinStateV1, Infallible>, I>
// where
//     I: InputPinV1<Error = Infallible>,
//     M: Monotonic,
//     M::Duration: Copy,
// {
//     fn is_high(&self) -> Result<bool, Self::Error> {
//         let input = &mut *self.0.borrow_mut();
//         Ok(input.read().unwrap_safe().stable() == PinStateV1::High)
//     }
//     fn is_low(&self) -> Result<bool, Self::Error> {
//         let input = &mut *self.0.borrow_mut();
//         Ok(input.read().unwrap_safe().stable() == PinStateV1::Low)
//     }
// }

#[cfg_attr(docsrs, doc(cfg(feature = "ehal0")))]
#[cfg(feature = "ehal0")]
impl<I: InputPinV0> Input<Result<PinStateV0, I::Error>> for I {
    fn read(&mut self) -> Result<PinStateV0, I::Error> {
        if self.is_high()? {
            Ok(PinStateV0::High)
        } else {
            Ok(PinStateV0::Low)
        }
    }
}

#[cfg_attr(docsrs, doc(cfg(feature = "ehal1")))]
#[cfg(feature = "ehal1")]
impl<I: InputPinV1> Input<Result<PinStateV1, I::Error>> for I {
    fn read(&mut self) -> Result<PinStateV1, I::Error> {
        if self.is_high()? {
            Ok(PinStateV1::High)
        } else {
            Ok(PinStateV1::Low)
        }
    }
}
