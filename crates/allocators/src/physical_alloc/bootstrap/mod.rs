//! Kernel box.
//! Implements allocation of kernel objects used to manage physical memory.

use spin::Mutex;

use crate::physical_alloc::KBox;

/// The physical address to start the kernel allocation.
static START_KALLOC: usize = 0xFFFF_FFFF;

/// The physical address to end the kernel allocation.
static END_KALLOC: usize = 0x1_FFFF_FFFF;

/// The size of the kernel allocation space.
static KALLOC_SIZE: usize = END_KALLOC - START_KALLOC;

/// The current physical address of the kernel allocation.
static CURR_KALLOC: Mutex<usize> = Mutex::new(START_KALLOC);

/// The error returned when the bootstrap allocator runs out of memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    OutOfMemory,
}

impl<T: 'static> KBox<T> {
    /// Allocates a kbox that holds the given type at bootstrap time.
    ///
    /// This uses the initial bump allocator to put the kbox at a constant offset from the start of the bootstrap allocation space.
    pub fn new_bootstrap(addr: T) -> Result<Self, Error> {
        // Find the size of the type.
        let size = core::mem::size_of::<T>();

        // Find the size of the alignment.
        let align = core::mem::align_of::<T>();

        // Get the next available address that is aligned.
        let mut curr_kalloc = CURR_KALLOC.lock();
        let next_addr = (*curr_kalloc + align - 1).div_euclid(align);

        // If the next address is greater than the end of the allocation space, return an error.
        if next_addr + size > END_KALLOC {
            return Err(Error::OutOfMemory);
        }

        // Allocate the memory.
        let ptr = unsafe { &mut *(next_addr as *mut T) };
        *ptr = addr;

        // Update the current physical address.
        *curr_kalloc = next_addr + size;

        // Return the kbox.
        Ok(Self { inner: ptr })
    }
}
