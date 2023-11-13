use crate::{InitializedValue, UninitializedValue, Value};

use rtic_time::Monotonic;

/// Represents a debouncer for handling signal noise in digital input signals.
/// It stabilizes the signal over a specified debounce period.
pub struct TimedDebouncer<M: Monotonic, T, V: Value<T = T> = InitializedValue<T>> {
    last_stable: V,
    last_value: V,
    last_change_time: M::Instant,
    debounce_time: M::Duration,
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
    pub fn stable_value(self: &Self) -> V::V {
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
    pub fn most_recent_value(self: &Self) -> V::V {
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

impl<M, T> TimedDebouncer<M, T, InitializedValue<T>>
where
    M: Monotonic,
    T: Copy,
    M::Duration: Copy,
{
    /// Creates a new Debouncer with a known initial value.
    pub fn new(initial_value: T, debounce_time: M::Duration) -> Self {
        Self {
            last_stable: initial_value.into(),
            last_value: initial_value.into(),
            last_change_time: M::now(),
            debounce_time,
        }
    }
}
impl<M, T> TimedDebouncer<M, T, UninitializedValue<T>>
where
    M: Monotonic,
    T: Copy + Default,
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
    V::V: Default + Copy + From<T>,
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
                stable: Default::default(),
                most_recent: Default::default(),
            }
        }
    }

    /// Reads the current stable value, if available. Potentially updating the internal state.
    pub fn read_value(&mut self) -> V::V {
        self.read().stable_value()
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
        fn set_compare(_instant: Self::Instant) {
            unimplemented!()
        }
        fn clear_compare_flag() {
            unimplemented!()
        }
        fn pend_interrupt() {
            unimplemented!()
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
