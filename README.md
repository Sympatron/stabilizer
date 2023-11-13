# Time-Based Debouncer

This crate provides a time-based debouncer for handling signal noise in digital input signals, particularly suited for embedded firmware applications. It stabilizes input signals over a specified debounce period, effectively filtering out noise and ensuring signal integrity.

## Features

- **Generic Implementation**: Works with any data type that implements the `PartialEq` and `Copy` traits.
- **Flexible Timing**: Utilizes the [`rtic_time::Monotonic`](https://docs.rs/rtic-time/latest/rtic_time/trait.Monotonic.html) trait for time handling, allowing for integration with various timer implementations.
- **State Tracking**: Tracks the current and previous states of the input, providing insight into signal transitions.

## Usage

First you have to provide an implementation of the [`rtic_time::Monotonic`](https://docs.rs/rtic-time/latest/rtic_time/trait.Monotonic.html) trait:

```rust
use fugit::ExtU32;
use rtic_monotonics::systick::*; // Replace with your Monotonic timer implementation

// Generate the required token. Replace this with something appropriate for your platform
let systick_token = rtic_monotonics::create_systick_token!();
// Start the monotonic
Systick::start(systick, 12_000_000, systick_token);
```
And here's a simple example of how to use the `Debouncer`:
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


