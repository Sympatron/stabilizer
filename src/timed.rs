use crate::{InitializedValue, Monotonic, State, UninitializedValue, Value};

/// Represents a debouncer for handling signal noise in digital input signals.
/// It stabilizes the signal over a specified debounce period.
pub struct TimedDebouncer<M: Monotonic, T, V: Value<T = T> = InitializedValue<T>> {
    last_stable: V,
    last_value: V,
    last_change_time: M::Instant,
    debounce_time: M::Duration,
}

impl<M, T> TimedDebouncer<M, T, InitializedValue<T>>
where
    M: Monotonic,
    T: Copy,
    M::Duration: Copy,
{
    /// Creates a new Debouncer with a known initial value.
    pub fn new(initial_value: T, debounce_time: M::Duration) -> Self {
        Self {
            last_stable: InitializedValue::new(initial_value),
            last_value: InitializedValue::new(initial_value),
            last_change_time: M::ZERO,
            debounce_time,
        }
    }
}
impl<M, T> TimedDebouncer<M, T, UninitializedValue<T>>
where
    M: Monotonic,
    T: Copy,
    M::Duration: Copy,
{
    /// Creates a new Debouncer that starts with an unkown state.
    pub fn new_unknown(debounce_time: M::Duration) -> Self {
        Self {
            last_stable: Default::default(),
            last_value: Default::default(),
            last_change_time: M::now(),
            debounce_time,
        }
    }
}
impl<M, T, V> TimedDebouncer<M, T, V>
where
    M: Monotonic,
    M::Duration: Copy,
    T: PartialEq + Copy,
    V: Value<T = T> + Copy + From<T>,
    V::V: Copy + From<T>,
{
    /// Updates the debouncer state with a new value and returns the current state.
    pub fn update(self: &mut Self, new_value: T) -> State<T, V> {
        if let Some(last_stable) = self.last_stable.try_get() {
            if last_stable == new_value {
                // value stayed stable or returned to stable
                self.last_value = new_value.into();
                return State::Stable { value: last_stable };
            }
        }
        if let Some(last_value) = self.last_value.try_get() {
            if last_value != new_value {
                // value changed since last update
                self.last_change_time = M::now();
            }
        } else {
            // first value
            self.last_change_time = M::now();
        }

        self.last_value = new_value.into();

        if M::now() >= self.last_change_time + self.debounce_time {
            // transitioned to a new state
            let last_stable = self.last_stable;
            self.last_stable = new_value.into();
            State::Transitioned {
                stable: new_value,
                previous_stable: *last_stable,
            }
        } else {
            // not stable at the moment
            State::Unstable {
                stable: *self.last_stable,
                most_recent: new_value.into(),
            }
        }
    }

    /// Reads the current state of the debouncer, updating it with the last known value.
    pub fn read(&mut self) -> State<T, V> {
        // Update the debouncer with the current value to potentially change its state.
        if let Some(last_value) = self.last_value.try_get() {
            self.update(last_value)
        } else {
            State::Unstable {
                stable: V::default(),
                most_recent: V::default(),
            }
        }
    }

    /// Reads the current stable value, if available. Potentially updating the internal state.
    pub fn read_value(&mut self) -> V::V {
        self.read().stable()
    }
    /// Reads the current stable value, if available. This does not update the internal state and just returns the last stable value.
    pub fn read_stable(&self) -> V::V {
        *self.last_stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fugit::ExtU64;
    extern crate std;

    struct MockMonotonic;
    static mut NOW: u64 = 0;
    static MUTEX: std::sync::Mutex<()> = std::sync::Mutex::new(());
    impl MockMonotonic {
        pub fn reset() {
            unsafe { NOW = 0 }
        }
        pub fn add(duration: <Self as Monotonic>::Duration) {
            unsafe { NOW += duration.ticks() }
        }
    }
    impl Monotonic for MockMonotonic {
        type Instant = fugit::TimerInstantU64<1_000_000>;
        type Duration = fugit::TimerDurationU64<1_000_000>;
        const ZERO: Self::Instant = Self::Instant::from_ticks(0);

        fn now() -> Self::Instant {
            if MUTEX.try_lock().is_ok() {
                panic!("Not locked");
            }
            unsafe { Self::Instant::from_ticks(NOW) }
        }
    }

    fn run_test(f: impl FnOnce(std::sync::MutexGuard<()>) -> ()) {
        let lock = MUTEX.lock().unwrap();
        MockMonotonic::reset();
        f(lock);
    }

    #[test]
    fn test_initial_value() {
        run_test(|_| {
            let mut debouncer = TimedDebouncer::<MockMonotonic, _>::new(false, 10.millis());
            assert_eq!(debouncer.read(), State::Stable { value: false });
            let state = debouncer.update(false);
            assert_eq!(state, State::Stable { value: false });
            let state = debouncer.update(true);
            assert_eq!(
                state,
                State::Unstable {
                    stable: false,
                    most_recent: true
                }
            );
            let state = debouncer.update(false);
            assert_eq!(state, State::Stable { value: false });
            debouncer.update(true);

            MockMonotonic::add(11.millis()); // Simulate time passing
            let state = debouncer.update(true);
            assert_eq!(
                state,
                State::Transitioned {
                    stable: true,
                    previous_stable: false
                }
            );
            let state = debouncer.update(true);
            assert_eq!(state, State::Stable { value: true });
        });
    }

    #[test]
    fn test_unknown_value() {
        run_test(|_| {
            let mut debouncer = TimedDebouncer::<MockMonotonic, _, _>::new_unknown(10.millis());
            assert_eq!(
                debouncer.read(),
                State::Unstable {
                    stable: None,
                    most_recent: None
                }
            );
            let state = debouncer.update(false);
            assert_eq!(
                state,
                State::Unstable {
                    stable: None,
                    most_recent: Some(false)
                }
            );
            MockMonotonic::add(11.millis()); // Simulate time passing
            let state = debouncer.update(false);
            assert_eq!(
                state,
                State::Transitioned {
                    stable: false,
                    previous_stable: None
                }
            );
            let state = debouncer.update(false);
            assert_eq!(state, State::Stable { value: false });
        });
    }
}
