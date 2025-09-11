use crate::address_translator::virtual_mem_table::{VirtualMemCellExt, VirtualMemCellOrLargePage};
use crate::{
    address_translator::{
        virtual_mem_table::{VirtualBasePage, VirtualMemCell},
        x86_types::{X86P1, X86P2, X86P3, X86P4},
    },
    kmalloc::KMalloc,
};

// 16 bits virtual address space
const ENTRIES_PER_TABLE: usize = 1 << 3;

const LARGE_PAGE_SIZE: usize = 1 << 8;

// 16 bits virtual address space + 6 bits for the table index = 22 bits for a full virtual address
const BASE_PAGE_SIZE: usize = 1 << 5;

const MEMZONE_ALLOC_SIZE: usize = 1 << 30;

pub type X86P4Test = X86P4<ENTRIES_PER_TABLE, LARGE_PAGE_SIZE, BASE_PAGE_SIZE>;
pub type X86P3Test = X86P3<ENTRIES_PER_TABLE, LARGE_PAGE_SIZE, BASE_PAGE_SIZE>;
pub type X86P2Test = X86P2<ENTRIES_PER_TABLE, LARGE_PAGE_SIZE, BASE_PAGE_SIZE>;
pub type X86P1Test = X86P1<ENTRIES_PER_TABLE, BASE_PAGE_SIZE>;

fn build_p1_table(kmalloc: &mut KMalloc) -> X86P1Test {
    const LEVEL_1_MULTIPLIER: usize = 2;

    let mut p1_table = X86P1Test::new(kmalloc);
    for i in 0..ENTRIES_PER_TABLE {
        let p0_page = VirtualBasePage::<BASE_PAGE_SIZE> {
            writable: true,
            readable: true,
        };

        p1_table
            .set_entry(
                i,
                VirtualMemCell {
                    phys_addr_offset: i * LEVEL_1_MULTIPLIER,
                    next_table: p0_page,
                },
            )
            .unwrap();
    }
    p1_table
}

fn build_p2_table(kmalloc: &mut KMalloc) -> X86P2Test {
    const LEVEL_2_MULTIPLIER: usize = 3;

    let mut p2_table = X86P2Test::new(kmalloc);
    for i in 0..ENTRIES_PER_TABLE {
        let p1_table = build_p1_table(kmalloc);
        p2_table
            .set_entry(
                i,
                VirtualMemCell {
                    phys_addr_offset: i * LEVEL_2_MULTIPLIER,
                    next_table: VirtualMemCellOrLargePage::PageTable(p1_table),
                },
            )
            .unwrap();
    }
    p2_table
}

fn build_p3_table(kmalloc: &mut KMalloc) -> X86P3Test {
    const LEVEL_3_MULTIPLIER: usize = 4;

    let mut p3_table = X86P3Test::new(kmalloc);
    for i in 0..ENTRIES_PER_TABLE {
        let p2_table = build_p2_table(kmalloc);
        p3_table
            .set_entry(
                i,
                VirtualMemCell {
                    phys_addr_offset: i * LEVEL_3_MULTIPLIER,
                    next_table: p2_table,
                },
            )
            .unwrap();
    }
    p3_table
}

fn build_p4_table(kmalloc: &mut KMalloc) -> X86P4Test {
    const LEVEL_4_MULTIPLIER: usize = 5;

    let mut p4_table = X86P4Test::new(kmalloc);
    for i in 0..ENTRIES_PER_TABLE {
        let p3_table = build_p3_table(kmalloc);
        p4_table
            .set_entry(
                i,
                VirtualMemCell {
                    phys_addr_offset: i * LEVEL_4_MULTIPLIER,
                    next_table: p3_table,
                },
            )
            .unwrap();
    }
    p4_table
}

#[test]
fn test_x86_translation() {
    // Spawn a new kmalloc to allocate the tables. We use a 1MB malloc memzone to be safe.
    let mut mem = vec![0_u8; MEMZONE_ALLOC_SIZE];

    // Let's get the address of the memzone.
    let memzone_addr = mem.as_mut_ptr() as usize;

    // Let's create the kernel memory allocator and set it for the memzone.
    let mut kmalloc = KMalloc::new(memzone_addr, memzone_addr + MEMZONE_ALLOC_SIZE);

    let p4_table = build_p4_table(&mut kmalloc);

    // Now let's translate some addresses.
    assert_eq!(p4_table.translate(0).unwrap(), 0);
    assert_eq!(p4_table.translate(1 << 18).unwrap(), 5 << 18);
    assert_eq!(
        p4_table.translate((1 << 18) + (1 << 14)).unwrap(),
        (5 << 18) + (4 << 14)
    );
    assert_eq!(
        p4_table
            .translate((1 << 18) + (1 << 14) + (1 << 10))
            .unwrap(),
        (5 << 18) + (4 << 14) + (3 << 10)
    );
    assert_eq!(
        p4_table
            .translate((1 << 18) + (1 << 14) + (1 << 10) + (1 << 6))
            .unwrap(),
        (5 << 18) + (4 << 14) + (3 << 10) + (2 << 6)
    );
}
