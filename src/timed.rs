use rtic_time::Monotonic;

/// Represents a debouncer for handling signal noise in digital input signals.
/// It stabilizes the signal over a specified debounce period.
pub struct TimedDebouncer<T, M: Monotonic> {
    last_stable: Option<T>,
    last_value: Option<T>,
    last_change_time: M::Instant,
    debounce_time: M::Duration,
}

/// Represents the state of a debounced input.
pub enum State<T> {
    /// Indicates a stable state with a known value.
    Stable {
        /// Current stable value.
        value: T,
    },
    /// Indicates an unstable state where the value is fluctuating.
    Unstable {
        /// Current stable value.
        stable: Option<T>,
        /// Most recent but potentially unstable value.
        current: Option<T>,
    },
    /// Indicates that a transition has occurred to a new stable state.
    Transitioned {
        /// New stable value after transition.
        new_stable: T,
        /// Old stable value before this transition.
        previous: Option<T>,
    },
}

impl<T: Copy> State<T> {
    /// Returns the current value of the state, if available.
    pub fn value(self: &Self) -> Option<T> {
        match self {
            State::Stable { value } => Some(*value),
            State::Unstable { stable, current: _ } => *stable,
            State::Transitioned {
                new_stable,
                previous: _,
            } => Some(*new_stable),
        }
    }

    /// Checks if the state has transitioned to a new value.
    pub fn transitioned(self: &Self) -> bool {
        match self {
            State::Transitioned {
                new_stable: _,
                previous: _,
            } => true,
            _ => false,
        }
    }
}

impl<T, M> TimedDebouncer<T, M>
where
    M: Monotonic,
    M::Duration: Copy,
    T: PartialEq + Copy,
{
    /// Creates a new Debouncer with a known initial value and debounce time.
    pub fn new(start_value: T, debounce_time: M::Duration) -> Self {
        Self {
            last_stable: Some(start_value),
            last_value: None,
            last_change_time: M::now(),
            debounce_time,
        }
    }

    /// Creates a new Debouncer without a known initial value.
    pub fn new_unknown(debounce_time: M::Duration) -> Self {
        Self {
            last_stable: None,
            last_value: None,
            last_change_time: M::now(),
            debounce_time,
        }
    }

    /// Updates the debouncer state with a new value and returns the current state.
    pub fn update(self: &mut Self, new_value: T) -> State<T> {
        if let Some(last_stable) = self.last_stable {
            if last_stable == new_value {
                // value stayed stable or returned to stable
                self.last_value = Some(new_value);
                return State::Stable { value: last_stable };
            }
        }
        if let Some(last_value) = self.last_value {
            if last_value != new_value {
                // value changed since last update
                self.last_change_time = M::now();
            }
        } else {
            // first value
            self.last_change_time = M::now();
        }

        self.last_value = Some(new_value);

        if M::now() >= self.last_change_time + self.debounce_time {
            // transitioned to a new state
            let last_stable = self.last_stable;
            self.last_stable = Some(new_value);
            State::Transitioned {
                new_stable: new_value,
                previous: last_stable,
            }
        } else {
            // not stable at the moment
            State::Unstable {
                stable: self.last_stable,
                current: Some(new_value),
            }
        }
    }

    /// Reads the current state of the debouncer, updating it with the last known value.
    pub fn read(&mut self) -> State<T> {
        // Update the debouncer with the current value to potentially change its state.
        if let Some(last_value) = self.last_value {
            self.update(last_value)
        } else {
            State::Unstable {
                stable: None,
                current: None,
            }
        }
    }

    /// Reads the current stable value, if available.
    pub fn read_value(&mut self) -> Option<T> {
        return self.read().value();
    }
}
