use spin::{Lazy, Mutex};

use crate::physical_alloc::{
    KBox,
    buddy::memzone::{AllocError, FreeError, MemZone, MemZone2M, MemZoneExt},
};

/// The physical memory allocator.
///
/// Is a kbox that is initialized on first access.
/// Since the Buddy allocator may require a lot of memory to be allocated, we
/// are better off using the kernel bump allocator, to potentially reallocate the
/// structures later on rather than using a static allocation.
static PHYSICAL_ALLOCATOR: Lazy<Mutex<KBox<BuddyAllocator>>> =
    Lazy::new(|| KBox::new_bootstrap(BuddyAllocator::new()).unwrap().into());

const NUM_ZONES: usize = 12;

pub struct BuddyAllocatorPage {
    zone: MemZone2M,
    start_addr: usize,
}

struct BuddyAllocator {
    pages: [BuddyAllocatorPage; NUM_ZONES],
}

pub enum BuddyAllocatorError {
    AllocationFailed,
    OffsetInvalid,
    FreeError(FreeError),
}

impl BuddyAllocator {
    const fn new() -> Self {
        Self {
            pages: [const {
                BuddyAllocatorPage {
                    zone: MemZone::Free,
                    // TODO: Map the physical address space and find the start addresses.
                    start_addr: 0,
                }
            }; NUM_ZONES],
        }
    }

    /// Allocates a memzone with the buddy allocator.
    /// Returns the address of the allocated memzone.
    fn alloc<T: MemZoneExt>(&mut self) -> Result<usize, BuddyAllocatorError> {
        // Iterate over the buddy allocator's zones.
        for page in self.pages.iter_mut() {
            // Try to allocate the zone.
            if let Ok(zone) = page.zone.alloc::<T>() {
                return Ok(zone + page.start_addr);
            }
        }

        Err(BuddyAllocatorError::AllocationFailed)
    }

    /// Frees a memzone with the buddy allocator.
    fn free<T: MemZoneExt>(&mut self, addr: usize) -> Result<(), BuddyAllocatorError> {
        // Find the memzone that contains the address.
        for page in self.pages.iter_mut() {
            if addr >= page.start_addr && addr < page.start_addr + page.zone.size() {
                page.zone
                    .free::<T>(addr - page.start_addr)
                    .map_err(BuddyAllocatorError::FreeError)?;
                return Ok(());
            }
        }

        Err(BuddyAllocatorError::OffsetInvalid)
    }
}
