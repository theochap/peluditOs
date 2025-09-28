//! Kernel reference counted pointer.
//! Implements allocation of kernel objects.

use core::ops::{Deref, DerefMut};

use crate::{kcell::KCell, utils::NonZero};

/// A kernel reference counted cell.
pub type KRefCell<T> = KRc<KCell<T>>;

/// A kernel reference counted cell.
///
/// This is a wrapper around a statically allocated value.
pub struct KRc<T: 'static> {
    /// The inner value. Needs to be mutable to be able to take ownership of the inner value
    /// when calling take().
    pub(super) inner: *mut NonZero<T>,
    pub(super) ref_count: &'static KCell<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KRcError {
    MoreThanOneRef,
}

impl<T: 'static> Deref for KRc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        let inner =
            unsafe { <*mut NonZero<T>>::as_ref(self.inner).expect("inner pointer cannot be null") };
        inner.inner_ref()
    }
}

impl<T: 'static> DerefMut for KRc<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        let inner = unsafe { self.inner.as_mut().expect("inner pointer cannot be null") };
        inner.inner_ref_mut()
    }
}

impl<T: 'static> Clone for KRc<T> {
    fn clone(&self) -> Self {
        self.ref_count.replace(|x| x + 1);
        Self {
            inner: self.inner,
            ref_count: self.ref_count,
        }
    }
}

impl<T: 'static> Drop for KRc<T> {
    fn drop(&mut self) {
        self.ref_count.replace(|x| x - 1);
        if *self.ref_count.get() == 1 {
            // TODO: free the object inside the memory allocator.
            let _ = self.inner;
        }
    }
}

impl<T: 'static> KRc<T> {
    /// Takes ownership of the inner value.
    ///
    /// Panics if the KRc is not unique.
    pub fn take(self) -> T {
        if *self.ref_count.get() > 1 {
            panic!("KRc is not unique");
        }

        unsafe {
            <*mut NonZero<T>>::as_mut(self.inner)
                .expect("inner pointer cannot be null")
                .take()
        }
    }
}
