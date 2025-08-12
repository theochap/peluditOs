//! Memory allocators for PeluditOS.
//!
//! Obviously this crate is no-std.

#![cfg_attr(any(not(test), target_os = "none"), no_std)]

pub mod karc;
pub mod kbox;
pub mod kmalloc;
pub mod kstack;
pub mod utils;

pub mod physical_alloc;
pub mod virtual_alloc;
