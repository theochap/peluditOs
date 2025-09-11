use crate::{kbox::KBox, kmalloc::KMalloc};

#[derive(Debug, PartialEq, Eq)]
pub enum AddressTranslationError {
    InvalidOffset {
        offset: usize,
        page_table_size: usize,
    },
    PageTableEntryNotFound(usize),
}

pub trait VirtualMemCellExt: 'static {
    const VIRTUAL_ADDRESS_BIT_OFFSET: usize;

    /// Translates a virtual address into a physical address.
    fn translate(&self, virtual_addr: usize) -> Result<usize, AddressTranslationError>;
}

pub struct VirtualBasePage<const SIZE: usize> {
    pub(super) writable: bool,
    pub(super) readable: bool,
}

const fn compute_num_bits_in_usize(mut size: usize) -> usize {
    let mut num_bits = 0;
    while size > 0 {
        size >>= 1;
        num_bits += 1;
    }
    num_bits
}

impl<const SIZE: usize> VirtualMemCellExt for VirtualBasePage<SIZE> {
    const VIRTUAL_ADDRESS_BIT_OFFSET: usize = compute_num_bits_in_usize(SIZE);

    fn translate(&self, offset: usize) -> Result<usize, AddressTranslationError> {
        Ok(offset)
    }
}

pub struct VirtualMemCell<Inner: VirtualMemCellExt> {
    pub(crate) phys_addr_offset: usize,
    pub(crate) next_table: Inner,
}

impl<Inner: VirtualMemCellExt> VirtualMemCellExt for VirtualMemCell<Inner> {
    const VIRTUAL_ADDRESS_BIT_OFFSET: usize = Inner::VIRTUAL_ADDRESS_BIT_OFFSET;

    #[inline(always)]
    fn translate(&self, offset: usize) -> Result<usize, AddressTranslationError> {
        Ok((self.phys_addr_offset << Inner::VIRTUAL_ADDRESS_BIT_OFFSET)
            | self.next_table.translate(offset)?)
    }
}

pub enum VirtualMemCellOrLargePage<const LARGE_PAGE_SIZE: usize, Inner: VirtualMemCellExt> {
    LargePage(VirtualBasePage<LARGE_PAGE_SIZE>),
    PageTable(Inner),
}

impl<const LARGE_PAGE_SIZE: usize, Inner: VirtualMemCellExt> From<VirtualBasePage<LARGE_PAGE_SIZE>>
    for VirtualMemCellOrLargePage<LARGE_PAGE_SIZE, Inner>
{
    fn from(value: VirtualBasePage<LARGE_PAGE_SIZE>) -> Self {
        Self::LargePage(value)
    }
}

impl<const LARGE_PAGE_SIZE: usize, Inner: VirtualMemCellExt> VirtualMemCellExt
    for VirtualMemCellOrLargePage<LARGE_PAGE_SIZE, Inner>
{
    const VIRTUAL_ADDRESS_BIT_OFFSET: usize = Inner::VIRTUAL_ADDRESS_BIT_OFFSET;

    fn translate(&self, offset: usize) -> Result<usize, AddressTranslationError> {
        match self {
            Self::LargePage(inner) => inner.translate(offset),
            Self::PageTable(inner) => inner.translate(offset),
        }
    }
}

pub struct VirtualMemTable<const NUM_ENTRIES: usize, Inner: VirtualMemCellExt>(
    KBox<[Option<VirtualMemCell<Inner>>; NUM_ENTRIES]>,
);

#[derive(Debug, PartialEq, Eq)]
pub enum SetEntryError {
    OutOfBounds { index: usize, num_entries: usize },
    InvalidPhysAddr { phys_addr: usize, bit_mask: usize },
}

impl<const NUM_ENTRIES: usize, Inner: VirtualMemCellExt> VirtualMemTable<NUM_ENTRIES, Inner> {
    const NUM_ENTRIES: usize = NUM_ENTRIES;

    pub fn new(kmalloc: &mut KMalloc) -> Self {
        Self(kmalloc.new_box([const { None }; NUM_ENTRIES]).unwrap())
    }

    pub fn set_entry(
        &mut self,
        index: usize,
        new_entry: VirtualMemCell<Inner>,
    ) -> Result<(), SetEntryError> {
        let Some(entry) = (*self.0).get_mut(index) else {
            return Err(SetEntryError::OutOfBounds {
                index,
                num_entries: NUM_ENTRIES,
            });
        };

        *entry = Some(new_entry);
        Ok(())
    }
}

impl<const NUM_ENTRIES: usize, Inner: VirtualMemCellExt> VirtualMemCellExt
    for VirtualMemTable<NUM_ENTRIES, Inner>
{
    const VIRTUAL_ADDRESS_BIT_OFFSET: usize =
        { Inner::VIRTUAL_ADDRESS_BIT_OFFSET + compute_num_bits_in_usize(NUM_ENTRIES) };

    fn translate(&self, offset: usize) -> Result<usize, AddressTranslationError> {
        // Translate the offset into a physical page offset.
        let phys_page_offset = offset >> Inner::VIRTUAL_ADDRESS_BIT_OFFSET;

        let entry = self
            .0
            .get(phys_page_offset)
            .ok_or(AddressTranslationError::InvalidOffset {
                offset: phys_page_offset,
                page_table_size: NUM_ENTRIES,
            })?
            .as_ref()
            .ok_or(AddressTranslationError::PageTableEntryNotFound(
                phys_page_offset,
            ))?;

        let inner_offset_mask: usize = (1 << Inner::VIRTUAL_ADDRESS_BIT_OFFSET) - 1;

        entry.translate(offset & inner_offset_mask)
    }
}
