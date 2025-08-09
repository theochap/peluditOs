//! Memory allocators for PeluditOS.
//!
//! Obviously this crate is no-std.

#![no_std]

pub mod physical_alloc;
pub mod virtual_alloc;
