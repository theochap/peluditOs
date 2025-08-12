use crate::{
    kmalloc::{Error, KMalloc},
    kstack::KStack,
    physical_alloc::{
        memmap::MemoryMap,
        memzone::{FreeError, MemZone, MemZone2M, MemZoneExt},
    },
};

const NUM_ZONES: usize = 12;

pub struct BuddyAllocatorPage {
    zone: MemZone2M,
    start_addr: usize,
}

struct BuddyAllocator {
    pages: KStack<BuddyAllocatorPage>,
    memmap: MemoryMap,
}

pub enum BuddyAllocatorError {
    AllocationFailed,
    OffsetInvalid,
    FreeError(FreeError),
}

impl BuddyAllocator {
    fn new(kbox_maker: &mut KMalloc, kbox_start: usize, memmap: MemoryMap) -> Result<Self, Error> {
        // Create the first page.
        Ok(Self {
            pages: KStack::new(
                kbox_maker,
                // We allocate the first 2MB page with a full zone. We assume that the bootstrap allocator
                // doesn't need more than that...
                BuddyAllocatorPage {
                    zone: MemZone::Full,
                    start_addr: kbox_start,
                },
            )?,
            memmap,
        })
    }

    /// Allocates a memzone with the buddy allocator.
    /// Returns the address of the allocated memzone.
    fn alloc<T: MemZoneExt>(
        &mut self,
        kbox_maker: &mut KMalloc,
    ) -> Result<usize, BuddyAllocatorError> {
        // Iterate over the buddy allocator's zones.
        let _ = self.pages.apply_until(|page| {
            // Try to allocate the zone.
            if let Ok(zone) = page.zone.alloc::<T>(kbox_maker) {
                return Ok::<_, ()>(Some(zone + page.start_addr));
            }

            Ok(None)
        });

        Err(BuddyAllocatorError::AllocationFailed)
    }

    /// Frees a memzone with the buddy allocator.
    fn free<T: MemZoneExt>(
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
