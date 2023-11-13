use core::marker::PhantomData;

use rtic_time::Monotonic;

/// Represents a debouncer for handling signal noise in digital input signals.
/// It stabilizes the signal over a specified debounce period.
pub struct TimedDebouncer<T, M: Monotonic, V = T> {
    last_stable: V,
    last_value: V,
    last_change_time: M::Instant,
    debounce_time: M::Duration,
    _t: PhantomData<T>,
}

/// Represents the state of a debounced input.
#[derive(Debug, PartialEq, Eq)]
pub enum State<T, V> {
    /// Indicates a stable state with a known value.
    Stable {
        /// Current stable value.
        value: T,
    },
    /// Indicates an unstable state where the value is fluctuating.
    Unstable {
        /// Current stable value.
        stable: V,
        /// Most recent but potentially unstable value.
        most_recent: V,
    },
    /// Indicates that a transition has occurred to a new stable state.
    Transitioned {
        /// New stable value after transition.
        stable: T,
        /// Old stable value before this transition.
        previous_stable: V,
    },
}

impl<T: Copy> State<T, T> {
    /// Returns the current value of the state
    pub fn stable_value(self: &Self) -> T {
        match self {
            State::Stable { value } => *value,
            State::Unstable {
                stable,
                most_recent: _,
            } => *stable,
            State::Transitioned {
                stable: new_stable,
                previous_stable: _,
            } => *new_stable,
        }
    }
    /// Returns the most recent value of the state. This value is potentially not stable yet.
    pub fn most_recent_value(self: &Self) -> T {
        match self {
            State::Stable { value } => *value,
            State::Unstable {
                stable: _,
                most_recent,
            } => *most_recent,
            State::Transitioned {
                stable: new_stable,
                previous_stable: _,
            } => *new_stable,
        }
    }
}
impl<T: Copy> State<T, Option<T>> {
    /// Returns the current stable value of the state, if available.
    pub fn stable_value(self: &Self) -> Option<T> {
        match self {
            State::Stable { value } => Some(*value),
            State::Unstable {
                stable,
                most_recent: _,
            } => *stable,
            State::Transitioned {
                stable: new_stable,
                previous_stable: _,
            } => Some(*new_stable),
        }
    }
    /// Returns the most recent value of the state, if available. This value is potentially not stable yet.
    pub fn most_recent_value(self: &Self) -> Option<T> {
        match self {
            State::Stable { value } => Some(*value),
            State::Unstable {
                stable: _,
                most_recent,
            } => *most_recent,
            State::Transitioned {
                stable: new_stable,
                previous_stable: _,
            } => Some(*new_stable),
        }
    }
}
impl<T, V> State<T, V> {
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

impl<T, M> TimedDebouncer<T, M, T>
where
    M: Monotonic,
    M::Duration: Copy,
    T: PartialEq + Copy,
{
    /// Creates a new Debouncer with a known initial value and debounce time.
    pub fn new(start_value: T, debounce_time: M::Duration) -> Self {
        Self {
            last_stable: start_value,
            last_value: start_value,
            last_change_time: M::now(),
            debounce_time,
            _t: PhantomData,
        }
    }
    /// Updates the debouncer state with a new value and returns the current state.
    pub fn update(self: &mut Self, new_value: T) -> State<T, T> {
        if self.last_stable == new_value {
            // value stayed stable or returned to stable
            self.last_value = new_value;
            return State::Stable {
                value: self.last_stable,
            };
        }
        if self.last_value != new_value {
            // value changed since last update
            self.last_change_time = M::now();
        }

        self.last_value = new_value;

        if M::now() >= self.last_change_time + self.debounce_time {
            // transitioned to a new state
            let last_stable = self.last_stable;
            self.last_stable = new_value;
            State::Transitioned {
                stable: new_value,
                previous_stable: last_stable,
            }
        } else {
            // not stable at the moment
            State::Unstable {
                stable: self.last_stable,
                most_recent: new_value,
            }
        }
    }

    /// Reads the current state of the debouncer, updating it with the last known value.
    pub fn read(&mut self) -> State<T, T> {
        // Update the debouncer with the current value to potentially change its state.
        self.update(self.last_value)
    }
    /// Reads the current stable value, if available.
    pub fn read_value(&mut self) -> T {
        return self.read().stable_value();
    }
    /// Reads the current stable value, if available. This does not update the internal state and just returns the last stable value.
    pub fn read_stable(&self) -> T {
        return self.last_stable;
    }
}

impl<T, M> TimedDebouncer<T, M, Option<T>>
where
    M: Monotonic,
    M::Duration: Copy,
    T: PartialEq + Copy,
{
    /// Creates a new Debouncer without a known initial value.
    pub fn new_unknown(debounce_time: M::Duration) -> Self {
        Self {
            last_stable: None,
            last_value: None,
            last_change_time: M::now(),
            debounce_time,
            _t: PhantomData,
        }
    }

    /// Updates the debouncer state with a new value and returns the current state.
    pub fn update(self: &mut Self, new_value: T) -> State<T, Option<T>> {
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
                stable: new_value,
                previous_stable: last_stable,
            }
        } else {
            // not stable at the moment
            State::Unstable {
                stable: self.last_stable,
                most_recent: Some(new_value),
            }
        }
    }

    /// Reads the current state of the debouncer, updating it with the last known value.
    pub fn read(&mut self) -> State<T, Option<T>> {
        // Update the debouncer with the current value to potentially change its state.
        if let Some(last_value) = self.last_value {
            self.update(last_value)
        } else {
            State::Unstable {
                stable: None,
                most_recent: None,
            }
        }
    }

    /// Reads the current stable value, if available.
    pub fn read_value(&mut self) -> Option<T> {
        return self.read().value();
    }
}
