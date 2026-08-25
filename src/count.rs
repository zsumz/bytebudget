//! Fixed-width byte quantity and architecture-boundary conversions.

use core::{convert::TryFrom, fmt};

/// An exact byte quantity in the crate's fixed `u64` accounting domain.
///
/// Arithmetic is available only through [`Self::checked_add`] and
/// [`Self::checked_sub`], keeping overflow and underflow explicit.
///
/// Arithmetic operators and implicit pointer-width conversions are absent.
///
/// ```compile_fail
/// use bytebudget::ByteCount;
/// let _ = ByteCount::new(1) + ByteCount::new(2);
/// ```
///
/// ```compile_fail
/// use bytebudget::ByteCount;
/// let _ = ByteCount::new(1).saturating_add(ByteCount::new(2));
/// ```
///
/// ```compile_fail
/// use bytebudget::ByteCount;
/// let _ = ByteCount::new(1).wrapping_sub(ByteCount::new(2));
/// ```
///
/// ```compile_fail
/// use bytebudget::ByteCount;
/// let _: ByteCount = 1_usize.into();
/// ```
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ByteCount(u64);

impl ByteCount {
    /// A count of zero bytes.
    pub const ZERO: Self = Self(0);

    /// The largest count representable by the accounting domain.
    pub const MAX: Self = Self(u64::MAX);

    /// Creates a count from an exact number of bytes.
    #[must_use]
    pub const fn new(bytes: u64) -> Self {
        Self(bytes)
    }

    /// Returns the count as an exact number of bytes.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// Returns whether the count is zero.
    #[must_use]
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Adds two counts, returning `None` when the sum exceeds [`Self::MAX`].
    #[must_use]
    pub const fn checked_add(self, other: Self) -> Option<Self> {
        match self.0.checked_add(other.0) {
            Some(bytes) => Some(Self(bytes)),
            None => None,
        }
    }

    /// Subtracts one count, returning `None` when it exceeds this count.
    #[must_use]
    pub const fn checked_sub(self, other: Self) -> Option<Self> {
        match self.0.checked_sub(other.0) {
            Some(bytes) => Some(Self(bytes)),
            None => None,
        }
    }
}

/// A byte-count conversion cannot preserve the source value exactly.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ByteCountOverflow;

impl fmt::Display for ByteCountOverflow {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("byte-count conversion would lose information")
    }
}

impl core::error::Error for ByteCountOverflow {}

impl From<u32> for ByteCount {
    fn from(bytes: u32) -> Self {
        Self(u64::from(bytes))
    }
}

impl From<u64> for ByteCount {
    fn from(bytes: u64) -> Self {
        Self(bytes)
    }
}

impl From<ByteCount> for u64 {
    fn from(count: ByteCount) -> Self {
        count.0
    }
}

impl TryFrom<usize> for ByteCount {
    type Error = ByteCountOverflow;

    fn try_from(bytes: usize) -> Result<Self, Self::Error> {
        match u64::try_from(bytes) {
            Ok(bytes) => Ok(Self(bytes)),
            Err(_) => Err(ByteCountOverflow),
        }
    }
}

impl TryFrom<ByteCount> for usize {
    type Error = ByteCountOverflow;

    fn try_from(count: ByteCount) -> Result<Self, Self::Error> {
        usize::try_from(count.0).map_err(|_| ByteCountOverflow)
    }
}
