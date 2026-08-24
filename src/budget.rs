//! Transactional aggregate accounting and its structured rejection errors.

use core::fmt;

use crate::ByteCount;

/// Aggregate byte accounting beneath one immutable limit.
///
/// A budget maintains `used <= limit`. Successful operations update `used`;
/// rejected operations leave all observable state unchanged.
///
/// A live budget is deliberately neither cloneable nor defaultable, and its
/// limit cannot be changed.
///
/// ```compile_fail
/// use bytebudget::{ByteBudget, ByteCount};
/// let budget = ByteBudget::new(ByteCount::new(8));
/// let _fork = budget.clone();
/// ```
///
/// ```compile_fail
/// use bytebudget::ByteBudget;
/// fn require_copy<T: Copy>() {}
/// require_copy::<ByteBudget>();
/// ```
///
/// ```compile_fail
/// use bytebudget::ByteBudget;
/// let _: ByteBudget = Default::default();
/// ```
///
/// ```compile_fail
/// use bytebudget::{ByteBudget, ByteCount};
/// let mut budget = ByteBudget::new(ByteCount::new(8));
/// budget.set_limit(ByteCount::new(16));
/// ```
#[must_use = "a byte budget has no effect unless it is retained and updated"]
#[derive(Debug, Eq, PartialEq)]
pub struct ByteBudget {
    limit: ByteCount,
    used: ByteCount,
}

impl ByteBudget {
    /// Creates an unused budget with the supplied fixed limit.
    pub const fn new(limit: ByteCount) -> Self {
        Self { limit, used: ByteCount::ZERO }
    }

    /// Returns the immutable byte limit.
    #[must_use]
    pub const fn limit(&self) -> ByteCount {
        self.limit
    }

    /// Returns the bytes currently reserved.
    #[must_use]
    pub const fn used(&self) -> ByteCount {
        self.used
    }

    /// Returns the bytes that can still be reserved.
    #[must_use]
    pub const fn available(&self) -> ByteCount {
        ByteCount::new(self.limit.get() - self.used.get())
    }

    /// Returns whether `amount` can be reserved without exceeding the limit.
    #[must_use]
    pub const fn can_reserve(&self, amount: ByteCount) -> bool {
        amount.get() <= self.available().get()
    }

    /// Returns whether no additional nonzero byte count can be reserved.
    #[must_use]
    pub const fn is_exhausted(&self) -> bool {
        self.used.get() == self.limit.get()
    }

    /// Reserves `amount` when it fits in the currently available capacity.
    ///
    /// On failure, the returned error captures both the attempted amount and
    /// the capacity available at rejection, and the budget remains unchanged.
    pub fn try_reserve(
        &mut self,
        amount: ByteCount,
    ) -> Result<(), CapacityExceeded> {
        let available = self.available();
        if self.can_reserve(amount) {
            self.used = ByteCount::new(self.used.get() + amount.get());
            Ok(())
        } else {
            Err(CapacityExceeded { requested: amount, available })
        }
    }

    /// Releases `amount` when no more than that amount is currently used.
    ///
    /// On failure, the returned error captures both the attempted release and
    /// the current use at rejection, and the budget remains unchanged.
    pub fn release(&mut self, amount: ByteCount) -> Result<(), OverRelease> {
        if amount.get() > self.used.get() {
            return Err(OverRelease { released: amount, used: self.used });
        }

        self.used = ByteCount::new(self.used.get() - amount.get());
        Ok(())
    }
}

/// A reservation was larger than the budget's available capacity.
///
/// Fields are private; use [`Self::requested`] and [`Self::available`].
///
/// ```compile_fail
/// use bytebudget::{ByteCount, CapacityExceeded};
/// let _ = CapacityExceeded {
///     requested: ByteCount::new(2),
///     available: ByteCount::new(1),
/// };
/// ```
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CapacityExceeded {
    requested: ByteCount,
    available: ByteCount,
}

impl CapacityExceeded {
    /// Returns the attempted reservation.
    #[must_use]
    pub const fn requested(&self) -> ByteCount {
        self.requested
    }

    /// Returns the capacity available when the reservation was rejected.
    #[must_use]
    pub const fn available(&self) -> ByteCount {
        self.available
    }
}

impl fmt::Display for CapacityExceeded {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "requested {} bytes, but only {} bytes are available",
            self.requested.get(),
            self.available.get()
        )
    }
}

impl core::error::Error for CapacityExceeded {}

/// A release was larger than the budget's current use.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OverRelease {
    released: ByteCount,
    used: ByteCount,
}

impl OverRelease {
    /// Returns the attempted release.
    #[must_use]
    pub const fn released(&self) -> ByteCount {
        self.released
    }

    /// Returns the bytes in use when the release was rejected.
    #[must_use]
    pub const fn used(&self) -> ByteCount {
        self.used
    }
}

impl fmt::Display for OverRelease {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "cannot release {} bytes when {} bytes are used",
            self.released.get(),
            self.used.get()
        )
    }
}

impl core::error::Error for OverRelease {}
