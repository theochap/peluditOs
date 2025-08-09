//! Physical memory allocator.
//!
//! This module contains the implementation of the physical memory allocator for PeluditOS.
//!
//! We're using a buddy system to manage the allocation of physical memory pages.

use core::{
    clone,
    error::Error,
    fmt::Debug,
    ops::{Deref, DerefMut},
};

/// The bootstrap allocator. Used to allocate most of the data structures necessary to bootstrap the buddy allocator.
mod bootstrap;

mod buddy;

/// A trait to define structs that hold allocated memory.
pub trait Boxed<T: 'static>:
    Deref<Target = T> + DerefMut<Target = T> + Sized + 'static + Debug
{
    type Error;

    fn new(val: T) -> Result<Self, Self::Error>;
}

/// A kernel box.
///
/// This is a wrapper around a statically allocated value.
/// It is used to manage the allocation of physical memory pages.
pub struct KBox<T: 'static> {
    inner: &'static mut T,
}

impl<T: 'static> From<&'static mut T> for KBox<T> {
    fn from(value: &'static mut T) -> Self {
        Self { inner: value }
    }
}

impl<T: 'static> Deref for KBox<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T: 'static> DerefMut for KBox<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
