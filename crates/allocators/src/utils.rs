use core::fmt::{self, Debug};

/// A non-zero value.
///
/// This is used to allow to retrieve the inner value from the KArc/KBox.
#[derive(PartialEq, Eq)]
pub enum NonZero<T> {
    Zero,
    NonZero(T),
}

impl<T: Debug> Debug for NonZero<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Zero => write!(f, "Zero"),
            Self::NonZero(value) => write!(f, "{value:?}"),
        }
    }
}

impl<T> Default for NonZero<T> {
    fn default() -> Self {
        Self::Zero
    }
}
