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
//! let mut debouncer = TimedDebouncer::<_, Systick>::new(false, 10.millis());
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

pub use timed::{State, TimedDebouncer};
pub(crate) use value::{InitializedValue, UninitializedValue, Value};
