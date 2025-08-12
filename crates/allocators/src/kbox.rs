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
#[derive(Debug)]
pub struct KBox<T: 'static> {
    pub(super) inner: &'static mut NonZero<T>,
}

impl<T: 'static> Drop for KBox<T> {
    fn drop(&mut self) {
        // TODO: free the object inside the memory allocator.
    }
}

impl<T: 'static> KBox<T> {
    pub fn take(self) -> T {
        let inner = mem::take(self.inner);
        match inner {
            NonZero::NonZero(inner) => inner,
            NonZero::Zero => unreachable!("KBox is zero"),
        }
    }
}

impl<T: 'static> Deref for KBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match &self.inner {
            NonZero::NonZero(inner) => inner,
            NonZero::Zero => unreachable!("KBox is zero"),
        }
    }
}

impl<T: 'static> DerefMut for KBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self.inner {
            NonZero::NonZero(inner) => inner,
            NonZero::Zero => unreachable!("KBox is zero"),
        }
    }
}
