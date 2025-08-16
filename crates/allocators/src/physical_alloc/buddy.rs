use crate::{
    kmalloc::{Error, KMalloc},
    kstack::KStack,
    physical_alloc::{
        memmap::{MemoryMap, MemoryMapKind},
        memzone::{
            AllocError, DEFAULT_MEM_CELL_SIZE, ErrorTypeExt, FreeError, MemZone, MemZone512,
            MemZoneExt,
        },
    },
};

#[derive(Debug, PartialEq, Eq)]
pub struct BuddyAllocatorPage<const SIZE: usize = DEFAULT_MEM_CELL_SIZE> {
    pub(crate) zone: MemZone512<SIZE>,
    pub(crate) start_addr: usize,
}

pub struct BuddyAllocator<const SIZE: usize = DEFAULT_MEM_CELL_SIZE> {
    pub(crate) pages: KStack<BuddyAllocatorPage<SIZE>>,
    memmap: MemoryMap,
    memmap_cursor: usize,
}

#[derive(Debug, PartialEq, Eq)]

pub enum BuddyAllocatorError {
    OutOfPhysicalMemoryPages,
    AllocationFailed(AllocError),
    OffsetInvalid,
    FreeError(FreeError),
    MallocError(Error),
}

impl<const SIZE: usize> BuddyAllocator<SIZE> {
    const PAGE_SIZE: usize = MemZone512::<SIZE>::SIZE;

    pub fn new(kbox_maker: &mut KMalloc, memmap: MemoryMap) -> Result<Self, Error> {
        // Create the first page.
        Ok(Self {
            pages: KStack::new(
                kbox_maker,
                // We allocate the first 2MB page with a full zone. We assume that the bootstrap allocator
                // doesn't need more than that...
                BuddyAllocatorPage {
                    zone: MemZone::Full,
                    start_addr: kbox_maker.start_addr,
                },
            )?,
            memmap,
            // We have allocated the first 2MB page so the cursor is at the next 2MB page.
            memmap_cursor: Self::PAGE_SIZE,
        })
    }

    fn try_allocate_new_zone(
        &mut self,
        kbox_maker: &mut KMalloc,
    ) -> Result<usize, BuddyAllocatorError> {
        let Some(memmap_head) = self.memmap.head() else {
            return Err(BuddyAllocatorError::OutOfPhysicalMemoryPages);
        };

        // Check if we have physical memory available.
        if self.memmap_cursor + Self::PAGE_SIZE > memmap_head.length {
            // We have no more physical memory available in the current physical memory zone.
            // We need to find the next usable physical memory zone.
            // We do this by iterating over the memory map.
            let mut found = false;
            while let Some(next_mem_entry) = self.memmap.pop() {
                if next_mem_entry.kind == MemoryMapKind::Usable
                    && next_mem_entry.length > Self::PAGE_SIZE
                {
                    found = true;
                    self.memmap_cursor = 0;
                    break;
                }
            }

            if !found {
                return Err(BuddyAllocatorError::OutOfPhysicalMemoryPages);
            }
        }

        let start_addr = self.memmap_cursor;

        // We have found a usable physical memory zone.
        // We need to add it to the buddy allocator.
        self.pages
            .push(
                kbox_maker,
                BuddyAllocatorPage {
                    zone: MemZone::Full,
                    start_addr,
                },
            )
            .map_err(BuddyAllocatorError::MallocError)?;

        self.memmap_cursor += Self::PAGE_SIZE;

        // If we get here, we have found a usable physical memory zone.
        Ok(start_addr)
    }

    /// Allocates a memzone with the buddy allocator.
    /// Returns the address of the allocated memzone.
    pub fn alloc<T: MemZoneExt<SIZE>>(
        &mut self,
        kbox_maker: &mut KMalloc,
    ) -> Result<usize, BuddyAllocatorError> {
        // Iterate over the buddy allocator's zones.
        match self.pages.apply_until(|page| {
            // Try to allocate the zone.
            match page.zone.alloc::<T>(kbox_maker) {
                Ok(zone) => Ok(Some(zone + page.start_addr)),
                Err(error) if error.is_fatal() => {
                    #[cfg(test)]
                    println!(
                        "Fatal error: Failed to allocate zone: {:?}. Error: {:?}",
                        page.zone, error
                    );
                    Err(error)
                }
                Err(error) => {
                    #[cfg(test)]
                    println!(
                        "Recoverable error: Failed to allocate zone: {:?}. Error: {:?}",
                        page.zone, error
                    );
                    Ok(None)
                }
            }
        }) {
            Ok(Some(iter_result)) => return Ok(iter_result),
            Err(err) => return Err(BuddyAllocatorError::AllocationFailed(err)),
            Ok(None) => {}
        }

        // If we get here, we should try to add a new allocation zone if there is still physical memory available.
        self.try_allocate_new_zone(kbox_maker)
    }

    /// Frees a memzone with the buddy allocator.
    fn free<T: MemZoneExt<SIZE>>(
        &mut self,
        kbox_maker: &mut KMalloc,
        addr: usize,
    ) -> Result<(), BuddyAllocatorError> {
        // Find the memzone that contains the address.
        self.pages.apply_until(|page| {
            if addr >= page.start_addr && addr < page.start_addr + page.zone.size() {
                page.zone
                    .free::<T>(addr - page.start_addr, kbox_maker)
                    .map_err(BuddyAllocatorError::FreeError)?;
                return Ok::<_, BuddyAllocatorError>(Some(()));
            }

            Ok(None)
        })?;

        Err(BuddyAllocatorError::OffsetInvalid)
    }
}
