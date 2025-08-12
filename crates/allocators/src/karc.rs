//! Kernel arc.
//! Implements allocation of kernel objects.

use core::{
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

/// A kernel arc.
///
/// This is a wrapper around a statically allocated value.
pub struct KArc<T: 'static> {
    /// The inner value.
    pub(super) inner: &'static T,
    pub(super) ref_count: &'static AtomicUsize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KArcError {
    MoreThanOneRef,
}

impl<T: 'static> Deref for KArc<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T: 'static> Clone for KArc<T> {
    fn clone(&self) -> Self {
        self.ref_count.fetch_add(1, Ordering::Relaxed);
        Self {
            inner: self.inner,
            ref_count: self.ref_count,
        }
    }
}

impl<T: 'static> Drop for KArc<T> {
    fn drop(&mut self) {
        if self.ref_count.fetch_sub(1, Ordering::SeqCst) == 1 {
            // TODO: free the object inside the memory allocator.
            drop(self.inner);
        }
    }
}
