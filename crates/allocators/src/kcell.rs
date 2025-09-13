//! Kernel implementation of a memory cell.

use core::cell::UnsafeCell;

use crate::utils::NonZero;

#[derive(Debug)]
pub struct KCell<T> {
    inner: UnsafeCell<NonZero<T>>,
}

impl<T> From<T> for KCell<T> {
    fn from(value: T) -> Self {
        Self::new(value)
    }
}

impl<T> KCell<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: UnsafeCell::new(inner.into()),
        }
    }

    pub fn set(&self, inner: T) {
        let ptr = unsafe {
            self.inner
                .get()
                .as_mut()
                .expect("this pointer cannot be null")
        };

        *ptr = inner.into();
    }

    pub fn replace(&self, f: impl FnOnce(T) -> T) {
        let ptr = unsafe {
            self.inner
                .get()
                .as_mut()
                .expect("this pointer cannot be null")
        };

        let val = ptr.take();

        *ptr = f(val).into();
    }

    pub fn get(&self) -> &T {
        // SAFETY: KCell is not sync, so we can't get a mutable reference to it if we already have a simple reference.
        unsafe {
            self.inner
                .get()
                .as_ref()
                .expect("this pointer cannot be null")
                .inner_ref()
        }
    }

    /// Takes back ownership of the inner value. Consumes the KCell.
    pub fn take(self) -> T {
        unsafe {
            self.inner
                .get()
                .as_mut()
                .expect("this pointer cannot be null")
                .take()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kcell() {
        let kcell = KCell::new(42);
        let kcell_ref = &kcell;

        assert_eq!(kcell.get(), &42);
        assert_eq!(kcell_ref.get(), &42);

        kcell.set(43);
        assert_eq!(kcell_ref.get(), &43);
        assert_eq!(kcell.get(), &43);
        assert_eq!(kcell_ref.get(), &43);

        kcell.replace(|x| x + 1);
        assert_eq!(kcell.get(), &44);
        assert_eq!(kcell_ref.get(), &44);
    }
}
