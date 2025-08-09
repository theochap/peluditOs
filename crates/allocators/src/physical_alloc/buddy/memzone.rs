use core::marker::PhantomData;

use crate::physical_alloc::{Boxed, KBox};

const MEM_CELL_SIZE: usize = 1 << 12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    MemZoneNotFree,
    MemZoneTooSmall {
        required_size: usize,
        available_size: usize,
    },
    OutOfMemory,
    SplitError(SplitError),
    ConsolidateError(ConsolidateError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitError {
    AllocError,
    CannotSplit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FreeError {
    ZoneAlreadyFree,
    InvalidOffset,
    SplitError(SplitError),
    ConsolidateError(ConsolidateError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConsolidateError {
    CannotConsolidate,
}

pub trait MemZoneExt: 'static {
    const SIZE: usize;

    /// Returns a mask that can be used to get the offset of a chunk of memory from the start of the memzone.
    const SIZE_MASK: usize = Self::SIZE - 1;

    /// Creates a new memzone with the given value.
    fn new(val: MemCell) -> Self;

    fn size(&self) -> usize {
        Self::SIZE
    }

    fn is_free(&self) -> bool;

    fn is_full(&self) -> bool;

    fn split(&mut self) -> Result<(), SplitError>;

    /// Allocates a chunk of memory from the memzone.
    ///
    /// Returns the offset of the allocated chunk compared to the start of the memzone.
    fn alloc<T: MemZoneExt>(&mut self) -> Result<usize, AllocError>;

    /// Frees a chunk of memory from the memzone.
    ///
    /// Is going to try to free the chunk of memory located at "offset".
    /// The offset is relative to the start of the memzone.
    ///
    /// ## Invariant
    /// This chunk of memory should always be full. We should traverse the memory zones down
    /// until we find a fully allocated chunk of memory.
    fn free<M: MemZoneExt>(&mut self, offset: usize) -> Result<(), FreeError>;

    /// Consolidates the memzone.
    ///
    /// This merges adjacent free/full chunks of memory.
    fn consolidate(&mut self) -> Result<(), ConsolidateError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum MemCell {
    Free,
    Full,
}

impl MemZoneExt for MemCell {
    const SIZE: usize = MEM_CELL_SIZE;

    fn new(val: MemCell) -> Self {
        val
    }

    fn is_free(&self) -> bool {
        matches!(self, Self::Free)
    }

    fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }

    fn split(&mut self) -> Result<(), SplitError> {
        Err(SplitError::CannotSplit)
    }

    fn alloc<T: MemZoneExt>(&mut self) -> Result<usize, AllocError> {
        if self.is_full() {
            return Err(AllocError::MemZoneNotFree);
        }

        if T::SIZE > Self::SIZE {
            return Err(AllocError::MemZoneTooSmall {
                required_size: T::SIZE,
                available_size: Self::SIZE,
            });
        }

        *self = Self::Full;

        Ok(0)
    }

    /// Frees a chunk of memory from the memzone.
    ///
    /// In this case, offset is always 0.
    fn free<M: MemZoneExt>(&mut self, offset: usize) -> Result<(), FreeError> {
        if offset != 0 {
            return Err(FreeError::InvalidOffset);
        }

        *self = Self::Free;
        Ok(())
    }

    /// Can never consolidate a single memcell.
    fn consolidate(&mut self) -> Result<(), ConsolidateError> {
        Err(ConsolidateError::CannotConsolidate)
    }
}

pub enum MemZone<T: MemZoneExt> {
    Free,
    Full,
    Partial { left: KBox<T>, right: KBox<T> },
}

impl<T: MemZoneExt> MemZoneExt for MemZone<T> {
    const SIZE: usize = 2 * T::SIZE;

    fn new(val: MemCell) -> Self {
        match val {
            MemCell::Free => MemZone::Free,
            MemCell::Full => MemZone::Full,
        }
    }

    fn is_free(&self) -> bool {
        matches!(self, Self::Free)
    }

    fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }

    fn split(&mut self) -> Result<(), SplitError> {
        // We cannot split a partial memzone.
        let inner = match self {
            Self::Partial { .. } => return Err(SplitError::CannotSplit),
            Self::Free => MemCell::Free,
            Self::Full => MemCell::Full,
        };

        let left = KBox::new_bootstrap(T::new(inner)).map_err(|_| SplitError::AllocError)?;
        let right = KBox::new_bootstrap(T::new(inner)).map_err(|_| SplitError::AllocError)?;
        *self = Self::Partial { left, right };
        Ok(())
    }

    fn alloc<M: MemZoneExt>(&mut self) -> Result<usize, AllocError> {
        if M::SIZE > Self::SIZE {
            return Err(AllocError::MemZoneTooSmall {
                required_size: M::SIZE,
                available_size: Self::SIZE,
            });
        }

        match self {
            Self::Full => Err(AllocError::MemZoneNotFree),

            Self::Partial { left, right, .. } => {
                let offset = match left.alloc::<M>() {
                    Ok(offset) => offset,
                    Err(_) => {
                        // If the left side failed, we need to allocate on the right side.
                        // We need to add the size of the left side to the offset to get the correct offset on the right side.
                        T::SIZE + right.alloc::<M>()?
                    }
                };

                // We need to consolidate the memzone to ensure that the left and right sides are not split.
                self.consolidate().map_err(AllocError::ConsolidateError)?;

                Ok(offset)
            }

            // If the memzone is free and the size is the same as the required size, we can allocate the whole memzone.
            Self::Free if M::SIZE == Self::SIZE => {
                *self = Self::Full;
                Ok(0)
            }

            // If the memzone is free, we need to split it and try again.
            Self::Free => {
                self.split().map_err(AllocError::SplitError)?;
                self.alloc::<M>()
            }
        }
    }

    fn free<M: MemZoneExt>(&mut self, offset: usize) -> Result<(), FreeError> {
        // We need to ensure the offset is valid.
        if offset & Self::SIZE_MASK != 0 {
            // If the offset doesn't fit in the memzone, it's invalid.
            return Err(FreeError::InvalidOffset);
        }

        match self {
            // If the memzone is free, we can't free anything.
            Self::Free => Err(FreeError::ZoneAlreadyFree),

            // If the memzone is full, we should
            // Compare the memzone size with the offset.
            // If the offset is null we should free the whole memzone.
            Self::Full if offset == 0 => {
                *self = Self::Free;
                Ok(())
            }

            Self::Partial { left, right, .. } => {
                if offset < T::SIZE {
                    left.free::<M>(offset)?;
                } else {
                    right.free::<M>(offset - T::SIZE)?;
                }

                self.consolidate().map_err(FreeError::ConsolidateError)?;

                Ok(())
            }

            // If the offset is lower than the memzone size, we should split the memzone and free the chunk of memory that matches the offset.
            Self::Full => {
                self.split().map_err(FreeError::SplitError)?;
                self.free::<M>(offset)
            }
        }
    }

    fn consolidate(&mut self) -> Result<(), ConsolidateError> {
        match self {
            Self::Free | Self::Full => Err(ConsolidateError::CannotConsolidate),

            Self::Partial { left, right, .. } if left.is_free() && right.is_free() => {
                *self = Self::Free;
                Ok(())
            }

            Self::Partial { left, right, .. } if left.is_full() && right.is_full() => {
                *self = Self::Full;
                Ok(())
            }

            _ => Ok(()),
        }
    }
}

type MemZone4K = MemZone<MemCell>;
type MemZone8K = MemZone<MemZone4K>;
type MemZone16K = MemZone<MemZone8K>;
type MemZone32K = MemZone<MemZone16K>;
type MemZone64K = MemZone<MemZone32K>;
type MemZone128K = MemZone<MemZone64K>;
type MemZone256K = MemZone<MemZone128K>;
type MemZone512K = MemZone<MemZone256K>;
type MemZone1M = MemZone<MemZone512K>;
pub(super) type MemZone2M = MemZone<MemZone1M>;
