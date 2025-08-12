//! Physical memory allocator.
//!
//! This module contains the implementation of the physical memory allocator for PeluditOS.
//!
//! We're using a buddy system to manage the allocation of physical memory pages.

mod buddy;
mod memmap;
mod memzone;

#[cfg(test)]
mod tests;
