/// A non-zero value.
///
/// This is used to allow to retrieve the inner value from the KArc/KBox.
#[derive(Debug)]
pub enum NonZero<T> {
    Zero,
    NonZero(T),
}

impl<T> Default for NonZero<T> {
    fn default() -> Self {
        Self::Zero
    }
}
