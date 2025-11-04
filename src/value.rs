use core::ops::Deref;

mod private {
    pub trait Sealed {}
}

/// Implementation detail to abstract away the differences between initialized and uninitialized debouncers
pub trait Value: Deref<Target = Self::V> + private::Sealed {
    /// The type of the value
    type T;
    /// The type of the value when it is initialized
    type V;
    /// Get the value
    fn get(&self) -> Self::V;
    /// Try to get the value if it is initialized
    fn try_get(&self) -> Option<Self::T>;
    /// Get the default value
    fn default() -> Self::V;
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct UninitializedValue<T>(Option<T>);
impl<T> private::Sealed for UninitializedValue<T> {}
impl<T: Copy> Value for UninitializedValue<T> {
    type T = T;
    type V = Option<T>;
    #[inline(always)]
    fn get(&self) -> Self::V {
        self.0
    }
    #[inline(always)]
    fn try_get(&self) -> Option<Self::T> {
        self.0
    }
    fn default() -> Self::V {
        None
    }
}
impl<T> Default for UninitializedValue<T> {
    fn default() -> Self {
        UninitializedValue(None)
    }
}
impl<T> Deref for UninitializedValue<T> {
    type Target = Option<T>;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> From<T> for UninitializedValue<T> {
    #[inline(always)]
    fn from(value: T) -> Self {
        UninitializedValue(Some(value))
    }
}
#[derive(Default, Clone, Copy, PartialEq, Eq, Debug)]
pub struct InitializedValue<T>(T);
impl<T> private::Sealed for InitializedValue<T> {}
impl<T: Copy> Value for InitializedValue<T> {
    type T = T;
    type V = T;
    #[inline(always)]
    fn get(&self) -> Self::V {
        self.0
    }
    #[inline(always)]
    fn try_get(&self) -> Option<Self::T> {
        Some(self.0)
    }
    #[inline(always)]
    fn default() -> Self::V {
        // Cannot be reached, beacause `try_get()` always returns `Some`
        unreachable!()
    }
}
impl<T> InitializedValue<T> {
    pub(crate) const fn new(value: T) -> Self {
        InitializedValue(value)
    }
}
impl<T> Deref for InitializedValue<T> {
    type Target = T;
    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> From<T> for InitializedValue<T> {
    #[inline(always)]
    fn from(value: T) -> Self {
        InitializedValue(value)
    }
}
