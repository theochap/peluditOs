//! Memory allocators for PeluditOS.
//!
//! Obviously this crate is no-std.

#![cfg_attr(not(test), no_std)]

pub mod structs;
pub(crate) use structs::*;

pub mod utils;

pub mod address_translator;
pub mod physical_alloc;
pub mod virtual_alloc;
