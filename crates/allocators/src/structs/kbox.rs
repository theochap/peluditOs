//! Kernel box.
//! Implements allocation of kernel objects.

use core::{
    mem,
    ops::{Deref, DerefMut},
};

use crate::utils::NonZero;

/// A kernel box.
///
/// This is a wrapper around a statically allocated value.
#[derive(Debug, PartialEq, Eq)]
pub struct KBox<T: 'static>(pub(super) &'static mut NonZero<T>);

#[cfg(test)]
impl<T: 'static> KBox<T> {
    pub fn inner(&self) -> &NonZero<T> {
        &self.0
    }
}

impl<T: 'static> Drop for KBox<T> {
    fn drop(&mut self) {
        // TODO: free the object inside the memory allocator.
    }
}

impl<T: 'static> KBox<T> {
    pub fn take(self) -> T {
        let inner = mem::take(self.0);
        match inner {
            NonZero::NonZero(inner) => inner,
            NonZero::Zero => unreachable!("KBox is zero"),
        }
    }

    pub fn try_map_inner_with_output<Out, F: FnOnce(T) -> Result<(T, Out), E>, E>(
        &mut self,
        f: F,
    ) -> Result<Out, E> {
        let inner = mem::take(self.0);
        let inner = match inner {
            NonZero::NonZero(inner) => inner,
            NonZero::Zero => unreachable!("KBox is zero"),
        };

        let (new_inner, out) = f(inner)?;

        *self.0 = NonZero::NonZero(new_inner);
        Ok(out)
    }

    /// Safely map the inner value of the KBox.
    pub fn map_inner_with_output<Out, F: FnOnce(T) -> (T, Out)>(&mut self, f: F) -> Out {
        self.try_map_inner_with_output(|inner| Ok::<(T, Out), ()>(f(inner)))
            .unwrap()
    }

    pub fn try_map_inner<F: FnOnce(T) -> Result<T, E>, E>(&mut self, f: F) -> Result<(), E> {
        self.try_map_inner_with_output(|inner| Ok::<(T, ()), E>((f(inner)?, ())))
    }

    /// Safely map the inner value of the KBox.
    pub fn map_inner<F: FnOnce(T) -> T>(&mut self, f: F) {
        self.try_map_inner(|inner| Ok::<T, ()>(f(inner))).unwrap();
    }
}

impl<T: 'static> Deref for KBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.0 {
            NonZero::NonZero(inner) => inner,
            NonZero::Zero => unreachable!("KBox is zero"),
        }
    }
}

impl<T: 'static> DerefMut for KBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.0 {
            NonZero::NonZero(inner) => inner,
            NonZero::Zero => unreachable!("KBox is zero"),
        }
    }
}
