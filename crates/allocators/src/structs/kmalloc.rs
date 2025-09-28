use core::ptr::{self};

use crate::{kbox::KBox, kcell::KCell, krc::KRc, utils::NonZero};

#[derive(Debug)]
pub struct KMalloc {
    /// The virtual start address of the allocation space.
    pub(crate) start_addr: usize,

    /// The current offset of the allocation space.
    pub(crate) offset: usize,

    /// The virtual end address of the allocation space.
    pub(crate) end_addr: usize,
}

/// The error returned when the allocator runs out of memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    OutOfMemory,
}

impl KMalloc {
    pub fn new(kalloc_start: usize, end_addr: usize) -> Self {
        Self {
            start_addr: kalloc_start,
            offset: 0,
            end_addr,
        }
    }

    /// Reserves memory for an array of the provided type and of size `num_elems`.
    #[inline(always)]
    pub fn reserve_array<T>(&mut self, num_elems: usize) -> Result<&'static mut [T], Error> {
        // Find the size of the type.
        let size = core::mem::size_of::<T>() * num_elems;

        // Find the size of the alignment.
        let align = core::mem::align_of::<T>();

        // Get the next available address that is aligned.
        let next_addr = (self.start_addr + self.offset + align - 1).div_euclid(align) * align;

        // If the next address is greater than the end of the allocation space, return an error.
        // The allocator should then bump the memory map to make more space.
        if next_addr + size > self.end_addr {
            return Err(Error::OutOfMemory);
        }

        // Allocate the memory.
        let fat: *mut [T] = ptr::slice_from_raw_parts_mut(next_addr as *mut T, num_elems);
        let ptr = unsafe { &mut *(fat as *mut [T]) };

        // Update the current physical address.
        self.offset += size;

        Ok(ptr)
    }

    /// Reserves memory for the given type but doesn't initialize it.
    #[inline(always)]
    fn reserve<T>(&mut self) -> Result<&'static mut T, Error> {
        // Find the size of the type.
        let size = core::mem::size_of::<T>();

        // Find the size of the alignment.
        let align = core::mem::align_of::<T>();

        // Get the next available address that is aligned.
        let next_addr = (self.start_addr + self.offset + align - 1).div_euclid(align) * align;

        // If the next address is greater than the end of the allocation space, return an error.
        // The allocator should then bump the memory map to make more space.
        if next_addr + size > self.end_addr {
            return Err(Error::OutOfMemory);
        }

        // Allocate the memory.
        let ptr = unsafe { &mut *(next_addr as *mut T) };

        // Update the current physical address.
        self.offset += size;

        Ok(ptr)
    }

    /// Allocates memory for the given type and initializes it with the given value.
    #[inline(always)]
    fn alloc<T: 'static>(&mut self, val: T) -> Result<&'static mut T, Error> {
        let ptr = self.reserve::<T>()?;

        *ptr = val;

        Ok(ptr)
    }

    /// Allocates a kbox that holds the given type at bootstrap time.
    ///
    /// This uses the initial bump allocator to put the kbox at a constant offset from the start of the bootstrap allocation space.
    pub fn new_box<T: 'static>(&mut self, addr: T) -> Result<KBox<T>, Error> {
        let ptr: &'static mut NonZero<T> = self.alloc(NonZero::NonZero(addr))?;

        // Return the kbox.
        Ok(KBox(ptr))
    }

    pub fn new_rc<T: 'static>(&mut self, addr: T) -> Result<KRc<T>, Error> {
        let ptr: *mut NonZero<T> = self.alloc(NonZero::NonZero(addr))?;

        let ref_count = KCell::new(1);
        let ref_ptr: &'static KCell<usize> = self.alloc(ref_count)?;

        Ok(KRc {
            inner: ptr,
            ref_count: ref_ptr,
        })
    }
}
