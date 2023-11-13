//! # Stabilizer
//!
//! The `stabilizer` crate provides a time-based debouncing implementation for embedded systems. Unlike traditional debouncers that rely on a fixed state polling interval, `stabilizer` offers a more dynamic approach. It allows for any polling interval and easily configurable debounce times, making it highly adaptable to various signal conditions and system requirements.
//!
//! ## Usage
//!
//! You can use it in your project as follows:
//!
//! ```ignore
//! use stabilizer::{TimedDebouncer, State};
//! use fugit::ExtU32;
//! use rtic_monotonics::systick::*; // Replace with your Monotonic timer implementation
//!
//! // Generate the required token
//! let systick_token = rtic_monotonics::create_systick_token!();
//! // Start the monotonic
//! Systick::start(systick, 12_000_000, systick_token);
//!
//! // Initialize the debouncer with a starting value and a debounce duration
//! let mut debouncer = TimedDebouncer::<Systick, _>::new(false, 10.millis());
//!
//! loop {
//!     // Update the debouncer with new values as they come in
//!     let state = debouncer.update(false);
//!
//!     // Check the state of the debouncer
//!     match state {
//!         State::Stable { value } => println!("Stable value: {:?}", value),
//!         State::Unstable { stable, most_recent } => println!("Unstable - Stable: {:?}, Current: {:?}", stable, most_recent),
//!         State::Transitioned { stable, previous_stable } => println!("Transitioned to {:?} from {:?}", stable, previous_stable),
//!     }
//!     delay(2.millis());
//! }
//! ```
//!
//! The crate is designed to be as generic as possible, working with any data type that implements `PartialEq` and `Copy`.
//!
//! ## Features
//!
//! - Time-based Debouncing: Flexible handling of signal noise by considering the time elapsed since the last signal change.
//! - Flexibility in Polling: Compatible with any polling interval, offering versatility for different system designs.
//! - Configurable Debounce Time: Easily adjustable debounce duration to suit various application needs.

#![no_std]
#![deny(missing_docs)]

mod timed;
mod value;
mod wrapper;

use core::ops::Add;

pub use timed::TimedDebouncer;
pub(crate) use value::{InitializedValue, UninitializedValue, Value};
pub use wrapper::{DebouncedInput, Input};

/// # Monotonic clock definition
///
/// If the feature `rtic-time` is enabled this will be automatically implemented for all `rtic_time::Monotonic`
pub trait Monotonic {
    /// The type for instant, defining an instant in time.
    type Instant: Ord + Copy + Add<Self::Duration, Output = Self::Instant>;
    /// The type for duration, defining an duration of time.
    type Duration;
    /// Get the current time.
    fn now() -> Self::Instant;
}
#[cfg(feature = "rtic-time")]
impl<M: rtic_time::Monotonic> Monotonic for M {
    type Instant = M::Instant;
    type Duration = M::Duration;
    fn now() -> Self::Instant {
        Self::now()
    }
}

/// Represents the state of a debounced input.
#[derive(Debug, PartialEq, Eq)]
pub enum State<T, V: Value<T = T>> {
    /// Indicates a stable state with a known value.
    Stable {
        /// Current stable value.
        value: T,
    },
    /// Indicates an unstable state where the value is fluctuating.
    Unstable {
        /// Current stable value.
        stable: V::V,
        /// Most recent but potentially unstable value.
        most_recent: V::V,
    },
    /// Indicates that a transition has occurred to a new stable state.
    Transitioned {
        /// New stable value after transition.
        stable: T,
        /// Old stable value before this transition.
        previous_stable: V::V,
    },
}

impl<T: Copy, V: Value<T = T>> State<T, V>
where
    V::V: Copy + From<T>,
{
    /// Returns the current stable value of the state, if available.
    pub fn stable(self: &Self) -> V::V {
        match self {
            State::Stable { value } => (*value).into(),
            State::Unstable {
                stable,
                most_recent: _,
            } => (*stable).into(),
            State::Transitioned {
                stable: new_stable,
                previous_stable: _,
            } => (*new_stable).into(),
        }
    }
    /// Returns the most recent value of the state, if available. This value is potentially not stable yet.
    pub fn most_recent(self: &Self) -> V::V {
        match self {
            State::Stable { value } => (*value).into(),
            State::Unstable {
                stable: _,
                most_recent,
            } => *most_recent,
            State::Transitioned {
                stable: new_stable,
                previous_stable: _,
            } => (*new_stable).into(),
        }
    }
}
impl<T, V: Value<T = T>> State<T, V> {
    /// Checks if the state has transitioned to a new value.
    pub fn transitioned(self: &Self) -> bool {
        match self {
            State::Transitioned {
                stable: _,
                previous_stable: _,
            } => true,
            _ => false,
        }
    }
}
