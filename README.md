# Time-Based Debouncer

This crate provides a time-based debouncer for handling signal noise in digital input signals, particularly suited for embedded firmware applications. It stabilizes input signals over a specified debounce period, effectively filtering out noise and ensuring signal integrity.

## Features

- **Generic Implementation**: Works with any data type that implements the `PartialEq` and `Copy` traits.
- **Flexible Timing**: Utilizes the `Monotonic` trait which is already implemented for all [`rtic_time::Monotonic`](https://docs.rs/rtic-time/latest/rtic_time/trait.Monotonic.html) for time handling, allowing for integration with any (global) timer implementations.
- **State Tracking**: Tracks the current and previous states of the input, providing insight into signal transitions.

## Usage

First you have to provide an implementation of the `Monotonic` trait. The easiest way is to provide any [`rtic_time::Monotonic`](https://docs.rs/rtic-time/latest/rtic_time/trait.Monotonic.html) trait implementation:

```rust
use fugit::ExtU32;
use rtic_monotonics::systick::*; // Replace with your Monotonic timer implementation

// Generate the required token. Replace this with something appropriate for your platform
let systick_token = rtic_monotonics::create_systick_token!();
// Start the monotonic
Systick::start(systick, 12_000_000, systick_token);
```
### Basic example
```rust
use stabilizer::TimedDebouncer;

// Initialize the debouncer with a starting value and a debounce duration
let mut debouncer = TimedDebouncer::<Systick, _>::new(false, 10.millis());

loop {
    // Update the debouncer with new values as they come in
    let state = debouncer.update(input.is_high());

    if state.transitioned() {
        let state = state.stable();
        //TODO Do something with the newly stable state
    }

    delay(2.millis());
}
```
### Advanced example
```rust
let mut debouncer = TimedDebouncer::<Systick, _>::new(false, 10.millis());
loop {
    // Update the debouncer with new values as they come in
    let state = debouncer.update(input.is_high());
    // Check the state of the debouncer
    match state {
        State::Stable { value } => println!("Stable value: {:?}", value),
        State::Unstable { stable, most_recent } => println!("Unstable - Stable: {:?}, Current: {:?}", stable, most_recent),
        State::Transitioned { stable, previous_stable } => println!("Transitioned to {:?} from {:?}", stable, previous_stable),
    }
    delay(2.millis());
}
```

## License

All source code (including code snippets) is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  [https://www.apache.org/licenses/LICENSE-2.0][L1])
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  [https://opensource.org/licenses/MIT][L2])

[L1]: https://www.apache.org/licenses/LICENSE-2.0
[L2]: https://opensource.org/licenses/MIT

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.


