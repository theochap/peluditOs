//! Virtual memory allocator.
//!
//! This module contains the implementation of the virtual memory allocator for PeluditOS.
//!
//! We're using a linked list of free pages to manage the allocation of virtual memory.

use crate::kstack::KStack;

pub struct VirtualMemPage {
    writable: bool,
    readable: bool,
    allocated: bool,

    size: usize,
}

pub struct VirtualMemAlloc {
    pub(crate) start_addr: usize,
    pub(crate) size: usize,
    pub(crate) cursor: usize,

    pages: KStack<VirtualMemPage>,
}
