use core::{
    fmt::{self, Debug},
    mem,
    ops::Deref,
};

/// A non-zero value.
///
/// This is used to allow to retrieve the inner value from the KArc/KBox.
#[derive(PartialEq, Eq)]
pub enum NonZero<T> {
    Zero,
    NonZero(T),
}

impl<T> From<T> for NonZero<T> {
    fn from(value: T) -> Self {
        Self::NonZero(value)
    }
}

impl<T: Debug> Debug for NonZero<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => write!(f, "Zero"),
            Self::NonZero(value) => write!(f, "{value:?}"),
        }
    }
}

impl<T> NonZero<T> {
    /// Returns a reference to the inner value.
    ///
    /// Panics if the NonZero is zero.
    pub fn inner_ref(&self) -> &T {
        match self {
            Self::NonZero(value) => &value,
            Self::Zero => panic!("Trying to deref a null NonZero!"),
        }
    }

    pub fn take(&mut self) -> T {
        let val = mem::take(self);

        match val {
            Self::NonZero(value) => value,
            Self::Zero => panic!("Trying to take a null NonZero!"),
        }
    }
}

impl<T> Default for NonZero<T> {
    fn default() -> Self {
        Self::Zero
    }
}
