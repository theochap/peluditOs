//! A simple module to translate between the virtual and physical address spaces

use core::usize;

mod virtual_mem_table;
mod x86_types;

#[cfg(test)]
mod tests;
