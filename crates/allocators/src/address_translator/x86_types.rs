use crate::address_translator::virtual_mem_table::{
    VirtualBasePage, VirtualMemCell, VirtualMemCellOrLargePage, VirtualMemTable,
};

/// By default 1024 entries per table on x86
const X86_ENTRIES_PER_TABLE: usize = 1 << 10;
/// 4kB base page size by default on x86
const BASE_PAGE_SIZE: usize = 1 << 12;
/// 2MB large page size on x86
const LARGE_PAGE_SIZE: usize = 1 << 21;

pub type X86P4<
    const NUM_ENTRIES_PER_TABLE: usize,
    const LARGE_PAGE_SIZE: usize,
    const BASE_PAGE_SIZE: usize,
> = VirtualMemTable<
    NUM_ENTRIES_PER_TABLE,
    X86P3<NUM_ENTRIES_PER_TABLE, LARGE_PAGE_SIZE, BASE_PAGE_SIZE>,
>;

pub type X86P3<
    const NUM_ENTRIES_PER_TABLE: usize,
    const LARGE_PAGE_SIZE: usize,
    const BASE_PAGE_SIZE: usize,
> = VirtualMemTable<
    NUM_ENTRIES_PER_TABLE,
    X86P2<NUM_ENTRIES_PER_TABLE, LARGE_PAGE_SIZE, BASE_PAGE_SIZE>,
>;

pub type X86P2<
    const NUM_ENTRIES_PER_TABLE: usize,
    const LARGE_PAGE_SIZE: usize,
    const BASE_PAGE_SIZE: usize,
> = VirtualMemTable<
    NUM_ENTRIES_PER_TABLE,
    VirtualMemCellOrLargePage<LARGE_PAGE_SIZE, X86P1<NUM_ENTRIES_PER_TABLE, BASE_PAGE_SIZE>>,
>;

pub type X86P1<const NUM_ENTRIES_PER_TABLE: usize, const BASE_PAGE_SIZE: usize> =
    VirtualMemTable<NUM_ENTRIES_PER_TABLE, VirtualBasePage<BASE_PAGE_SIZE>>;
