use crate::kmalloc::KMalloc;
use crate::physical_alloc::buddy::BuddyAllocator;
use crate::physical_alloc::memmap::{MemoryMap, MemoryMapEntry, MemoryMapKind};
use crate::physical_alloc::memzone::{MemZone, MemZone128, MemZone256, MemZone512, MemZoneExt};
use crate::utils::NonZero;

// Let's allocate a 4KB memzone.
const MEMZONE_SIZE: usize = 4096;
const NUM_PAGES: usize = 4 * 512 * MEMZONE_SIZE;

/// Tries to allocate a kbox on an existing memzone. Ensure that the kbox is
/// allocated in the memzone and that the data is stored correctly.
#[test]
fn test_allocate_memzone() {
    let mut mem = vec![0_u8; MEMZONE_SIZE];

    // Let's get the address of the memzone.
    let memzone_addr = mem.as_mut_ptr() as usize;

    // Let's create the kernel memory allocator and set it for the memzone.
    let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + MEMZONE_SIZE);

    let initial_addr = kmalloc.start_addr;
    let final_addr = memzone_addr + MEMZONE_SIZE;
    assert_eq!(initial_addr, memzone_addr);
    assert_eq!(final_addr, initial_addr + MEMZONE_SIZE);

    let string = "hello world!";
    let expected_value = NonZero::NonZero(string);

    let value_size = core::mem::size_of::<NonZero<&str>>();
    let align = core::mem::align_of::<NonZero<&str>>();

    // Let's allocate a kernel box with some data in it.
    let kbox = kmalloc.new_box(string).unwrap();

    assert_eq!(kmalloc.offset, value_size);
    assert_eq!(kmalloc.offset % align, 0);

    assert_eq!(initial_addr, kmalloc.start_addr);
    assert_eq!(final_addr, kmalloc.end_addr);

    let mem_ptr = mem.as_ptr() as *const NonZero<&str>;
    let mem_bytes: &NonZero<&str> = unsafe { &*mem_ptr };

    assert_eq!(mem_bytes, &expected_value);
    assert_eq!(kbox.inner(), &expected_value);
}

pub struct BuddyAllocatorSetup {
    pub buddy_allocator: BuddyAllocator<4096>,
    pub kmalloc: KMalloc,
    pub mem: Vec<u8>,
}

/// Note: we have to return the last parameter because we need to keep the heap
/// allocated for the test.
fn setup_buddy_allocator() -> BuddyAllocatorSetup {
    let mut mem = vec![0_u8; NUM_PAGES * MEMZONE_SIZE];

    // Let's get the address of the memzone.
    let memzone_addr = mem.as_mut_ptr() as usize;

    // Let's create the kernel memory allocator and set it for the memzone.
    let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + MEMZONE_SIZE);

    let memmap = MemoryMap::new(
        MemoryMapEntry {
            base_addr: memzone_addr,
            length: NUM_PAGES * MEMZONE_SIZE,
            kind: MemoryMapKind::Usable,
        },
        &mut kmalloc,
    )
    .unwrap();

    // Now create a buddy allocator. It should be able to hold 4 full zones.
    let buddy_allocator = BuddyAllocator::<MEMZONE_SIZE>::new(&mut kmalloc, memmap).unwrap();

    BuddyAllocatorSetup {
        buddy_allocator,
        kmalloc,
        mem,
    }
}

#[test]
fn memmap_structure() {
    let BuddyAllocatorSetup {
        buddy_allocator,
        kmalloc,
        mem: _mem,
    } = setup_buddy_allocator();

    assert_eq!(buddy_allocator.memmap_cursor, 512 * MEMZONE_SIZE);

    assert_eq!(
        buddy_allocator.memmap.head().unwrap().base_addr,
        kmalloc.start_addr
    );
    assert_eq!(
        buddy_allocator.memmap.head().unwrap().length,
        NUM_PAGES * MEMZONE_SIZE
    );
    assert_eq!(
        buddy_allocator.memmap.head().unwrap().kind,
        MemoryMapKind::Usable
    );

    assert_eq!(
        buddy_allocator.pages.head().unwrap().start_addr,
        kmalloc.start_addr
    );
    assert_eq!(
        buddy_allocator.pages.head().unwrap().zone.size(),
        512 * MEMZONE_SIZE
    );
    assert!(buddy_allocator.pages.head().unwrap().zone.is_full());
}

/// Tries to allocate a memzone using the buddy allocator.
#[test]
fn test_buddy_allocator_alloc() {
    let BuddyAllocatorSetup {
        mut buddy_allocator,
        mut kmalloc,
        mem: _mem,
    } = setup_buddy_allocator();

    assert_eq!(buddy_allocator.memmap_cursor, 512 * MEMZONE_SIZE);

    let mut iterator = buddy_allocator.pages.iter();
    let first_page = iterator.next().unwrap();
    assert!(first_page.zone.is_full());
    assert_eq!(first_page.zone.size(), 512 * MEMZONE_SIZE);
    assert_eq!(first_page.start_addr, kmalloc.start_addr);

    assert_eq!(iterator.next(), None);

    // Let's allocate the full second zone.
    let addr = buddy_allocator
        .alloc::<MemZone512<MEMZONE_SIZE>>(&mut kmalloc)
        .unwrap();

    assert_eq!(buddy_allocator.memmap_cursor, 1024 * MEMZONE_SIZE);

    // Iterate over the pages and ensure that the new page is full.
    // We are currently using a stack and not a queue, so the first page is the one
    // that was allocated last.
    let mut iterator = buddy_allocator.pages.iter();
    let first_page = iterator.next().unwrap();
    assert!(first_page.zone.is_full());
    assert_eq!(first_page.zone.size(), 512 * MEMZONE_SIZE);
    assert_eq!(
        first_page.start_addr,
        kmalloc.start_addr + 512 * MEMZONE_SIZE
    );

    let second_page = iterator.next().unwrap();
    assert!(second_page.zone.is_full());
    assert_eq!(second_page.zone.size(), 512 * MEMZONE_SIZE);
    assert_eq!(second_page.start_addr, kmalloc.start_addr);

    assert_eq!(iterator.next(), None);
}

#[test]
fn test_buddy_allocator_split() {
    let BuddyAllocatorSetup {
        mut buddy_allocator,
        mut kmalloc,
        mem: _mem,
    } = setup_buddy_allocator();

    // Let's allocate half a full zone.
    let addr = buddy_allocator
        .alloc::<MemZone256<MEMZONE_SIZE>>(&mut kmalloc)
        .unwrap();

    // Iterate over the pages and ensure that the second page is only half full.
    let mut iterator = buddy_allocator.pages.iter();

    let second_page = iterator.next().unwrap();

    match &second_page.zone {
        MemZone::Partial { left, right } => {
            assert!(left.is_full());
            assert!(right.is_free());
        }
        page => panic!("New page is not partial: {:?}", page),
    }

    let first_page = iterator.next().unwrap();
    assert!(first_page.zone.is_full());
    assert_eq!(first_page.zone.size(), 512 * MEMZONE_SIZE);
}

#[test]
fn test_buddy_alloc_free_with_consolidate() {
    let BuddyAllocatorSetup {
        mut buddy_allocator,
        mut kmalloc,
        mem: _mem,
    } = setup_buddy_allocator();

    // Let's allocate half a full zone.
    let half_full_addr = buddy_allocator
        .alloc::<MemZone256<MEMZONE_SIZE>>(&mut kmalloc)
        .unwrap();

    // Let's allocate a quarter full zone
    let quarter_addr = buddy_allocator
        .alloc::<MemZone128<MEMZONE_SIZE>>(&mut kmalloc)
        .unwrap();

    // Ensure the top zone is double splitted
    let mut iterator = buddy_allocator.pages.iter();

    let second_page = iterator.next().unwrap();

    match &second_page.zone {
        MemZone::Partial { left, right } => {
            assert!(left.is_full());

            match &**right {
                MemZone::Partial {
                    left: inner_left,
                    right: inner_right,
                } => {
                    assert!(inner_left.is_full());
                    assert!(inner_right.is_free());
                }
                _ => {
                    panic!("Inner memzone should be partial");
                }
            }
        }
        page => panic!("New page is not partial: {page:?}"),
    }

    // Free the half full zone
    buddy_allocator
        .free::<MemZone256<MEMZONE_SIZE>>(&mut kmalloc, half_full_addr)
        .unwrap();

    // Let's check that only the right zone is allocated
    let mut iterator = buddy_allocator.pages.iter();

    let second_page = iterator.next().unwrap();

    match &second_page.zone {
        MemZone::Partial { left, right } => {
            assert!(left.is_free());

            match &**right {
                MemZone::Partial {
                    left: inner_left,
                    right: inner_right,
                } => {
                    assert!(inner_left.is_full());
                    assert!(inner_right.is_free());
                }
                _ => {
                    panic!("Inner memzone should be partial");
                }
            }
        }
        page => panic!("New page is not partial: {page:?}"),
    }

    // Free both zones
    buddy_allocator
        .free::<MemZone128<MEMZONE_SIZE>>(&mut kmalloc, quarter_addr)
        .unwrap();

    // The first zone should be free
    let mut iterator = buddy_allocator.pages.iter();

    let second_page = iterator.next().unwrap();
    assert!(second_page.zone.is_free());
}
