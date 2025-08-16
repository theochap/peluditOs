use crate::{kbox::KBox, kmalloc::KMalloc};

pub const DEFAULT_MEM_CELL_SIZE: usize = 1 << 12;

/// The type of error that occurred during an allocation.
///
/// This is used to determine if the allocation should be retried or if the error should be propagated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Continue,
    Fatal,
}

pub trait ErrorTypeExt {
    fn error_severity(&self) -> ErrorSeverity;

    fn is_fatal(&self) -> bool {
        matches!(self.error_severity(), ErrorSeverity::Fatal)
    }

    fn is_continue(&self) -> bool {
        matches!(self.error_severity(), ErrorSeverity::Continue)
    }
}

impl ErrorTypeExt for AllocError {
    fn error_severity(&self) -> ErrorSeverity {
        match self {
            AllocError::MemZoneNotFree => ErrorSeverity::Continue,
            AllocError::MemZoneTooSmall { .. } => ErrorSeverity::Continue,
            AllocError::OutOfMemory => ErrorSeverity::Fatal,
            AllocError::SplitError(_) => ErrorSeverity::Fatal,
        }
    }
}

impl ErrorTypeExt for FreeError {
    fn error_severity(&self) -> ErrorSeverity {
        match self {
            FreeError::ZoneAlreadyFree => ErrorSeverity::Fatal,
            FreeError::InvalidOffset => ErrorSeverity::Fatal,
            FreeError::SplitError(_) => ErrorSeverity::Fatal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllocError {
    MemZoneNotFree,
    MemZoneTooSmall {
        required_size: usize,
        available_size: usize,
    },
    OutOfMemory,
    SplitError(SplitError),
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
}

pub trait MemZoneExt<const BASE: usize = DEFAULT_MEM_CELL_SIZE>: 'static {
    const SIZE: usize;

    /// Returns a mask that can be used to get the offset of a chunk of memory from the start of the memzone.
    const SIZE_MASK: usize = Self::SIZE - 1;

    /// Creates a new memzone with the given value.
    fn new(is_free: bool) -> Self;

    fn size(&self) -> usize {
        Self::SIZE
    }

    fn is_free(&self) -> bool;

    fn is_full(&self) -> bool;

    fn split(&mut self, kbox_maker: &mut KMalloc) -> Result<(), SplitError>;

    /// Allocates a chunk of memory from the memzone.
    ///
    /// Returns the offset of the allocated chunk compared to the start of the memzone.
    fn alloc<T: MemZoneExt<BASE>>(&mut self, kbox_maker: &mut KMalloc)
    -> Result<usize, AllocError>;

    /// Frees a chunk of memory from the memzone.
    ///
    /// Is going to try to free the chunk of memory located at "offset".
    /// The offset is relative to the start of the memzone.
    ///
    /// ## Invariant
    /// This chunk of memory should always be full. We should traverse the memory zones down
    /// until we find a fully allocated chunk of memory.
    fn free<M: MemZoneExt<BASE>>(
        &mut self,
        offset: usize,
        kbox_maker: &mut KMalloc,
    ) -> Result<(), FreeError>;

    /// Consolidates the memzone.
    ///
    /// This merges adjacent free/full chunks of memory.
    fn consolidate(&mut self);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub enum MemCell<const SIZE: usize = DEFAULT_MEM_CELL_SIZE> {
    Free,
    Full,
}

impl<const SIZE: usize> MemZoneExt<SIZE> for MemCell<SIZE> {
    const SIZE: usize = SIZE;

    fn new(free: bool) -> Self {
        if free { Self::Free } else { Self::Full }
    }

    fn is_free(&self) -> bool {
        matches!(self, Self::Free)
    }

    fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }

    fn split(&mut self, _kbox_maker: &mut KMalloc) -> Result<(), SplitError> {
        Err(SplitError::CannotSplit)
    }

    fn alloc<T: MemZoneExt<SIZE>>(
        &mut self,
        _kbox_maker: &mut KMalloc,
    ) -> Result<usize, AllocError> {
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
    fn free<M: MemZoneExt<SIZE>>(
        &mut self,
        offset: usize,
        _kbox_maker: &mut KMalloc,
    ) -> Result<(), FreeError> {
        if offset != 0 {
            return Err(FreeError::InvalidOffset);
        }

        *self = Self::Free;
        Ok(())
    }

    /// Can never consolidate a single memcell.
    fn consolidate(&mut self) {
        // Do nothing.
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MemZone<const BASE: usize, T: MemZoneExt<BASE>> {
    Free,
    Full,
    Partial { left: KBox<T>, right: KBox<T> },
}

impl<const BASE: usize, T: MemZoneExt<BASE>> MemZoneExt<BASE> for MemZone<BASE, T> {
    const SIZE: usize = 2 * T::SIZE;

    fn new(free: bool) -> Self {
        if free {
            MemZone::<BASE, T>::Free
        } else {
            MemZone::<BASE, T>::Full
        }
    }

    fn is_free(&self) -> bool {
        matches!(self, Self::Free)
    }

    fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }

    fn split(&mut self, kbox_maker: &mut KMalloc) -> Result<(), SplitError> {
        let left = kbox_maker
            .new_box(T::new(self.is_free()))
            .map_err(|_| SplitError::AllocError)?;
        let right = kbox_maker
            .new_box(T::new(self.is_free()))
            .map_err(|_| SplitError::AllocError)?;
        *self = Self::Partial { left, right };
        Ok(())
    }

    fn alloc<M: MemZoneExt<BASE>>(
        &mut self,
        kbox_maker: &mut KMalloc,
    ) -> Result<usize, AllocError> {
        if M::SIZE > Self::SIZE {
            return Err(AllocError::MemZoneTooSmall {
                required_size: M::SIZE,
                available_size: Self::SIZE,
            });
        }

        match self {
            Self::Full => Err(AllocError::MemZoneNotFree),

            Self::Partial { left, right, .. } => {
                let offset = match left.alloc::<M>(kbox_maker) {
                    Ok(offset) => offset,
                    Err(_) => {
                        // If the left side failed, we need to allocate on the right side.
                        // We need to add the size of the left side to the offset to get the correct offset on the right side.
                        T::SIZE + right.alloc::<M>(kbox_maker)?
                    }
                };

                // We need to consolidate the memzone to ensure that the left and right sides are not split.
                self.consolidate();

                Ok(offset)
            }

            // If the memzone is free and the size is the same as the required size, we can allocate the whole memzone.
            Self::Free if M::SIZE == Self::SIZE => {
                *self = Self::Full;
                Ok(0)
            }

            // If the memzone is free, we need to split it and try again.
            Self::Free => {
                self.split(kbox_maker).map_err(AllocError::SplitError)?;
                self.alloc::<M>(kbox_maker)
            }
        }
    }

    fn free<M: MemZoneExt<BASE>>(
        &mut self,
        offset: usize,
        kbox_maker: &mut KMalloc,
    ) -> Result<(), FreeError> {
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
                    left.free::<M>(offset, kbox_maker)?;
                } else {
                    right.free::<M>(offset - T::SIZE, kbox_maker)?;
                }

                self.consolidate();

                Ok(())
            }

            // If the offset is lower than the memzone size, we should split the memzone and free the chunk of memory that matches the offset.
            Self::Full => {
                self.split(kbox_maker).map_err(FreeError::SplitError)?;
                self.free::<M>(offset, kbox_maker)
            }
        }
    }

    fn consolidate(&mut self) {
        match self {
            Self::Partial { left, right, .. } if left.is_free() && right.is_free() => {
                *self = Self::Free;
            }

            Self::Partial { left, right, .. } if left.is_full() && right.is_full() => {
                *self = Self::Full;
            }

            _ => {
                // Do nothing.
            }
        }
    }
}

macro_rules! define_memzone_types {
    (
        base: $base_name:ident<$base_type:ident>,
        types: [
            $($name:ident<$inner:ident>),* $(,)?
        ],
        pub: $pub_name:ident<$pub_inner:ident>
    ) => {
        type $base_name<const SIZE: usize = DEFAULT_MEM_CELL_SIZE> = MemZone<SIZE, $base_type<SIZE>>;
        $(
            type $name<const SIZE: usize = DEFAULT_MEM_CELL_SIZE> = MemZone<SIZE, $inner<SIZE>>;
        )*
        pub(super) type $pub_name<const SIZE: usize = DEFAULT_MEM_CELL_SIZE> =
            MemZone<SIZE, $pub_inner<SIZE>>;
    };
}

define_memzone_types! {
    base: MemZone2<MemCell>,
    types: [
        MemZone4<MemZone2>,
        MemZone8<MemZone4>,
        MemZone16<MemZone8>,
        MemZone32<MemZone16>,
        MemZone64<MemZone32>,
        MemZone128<MemZone64>,
        MemZone256<MemZone128>,
    ],
    pub: MemZone512<MemZone256>
}
