//! Consumer-owned retained-storage measurement contract.

use crate::ByteCount;

/// Reports variable storage retained by one value under an implementation's
/// documented accounting model.
///
/// An implementation must:
///
/// - exclude `size_of::<Self>()` and fixed queue-slot or envelope overhead;
/// - include transitively owned variable backing storage;
/// - charge allocated capacity, not logical length, when capacity remains
///   allocated;
/// - never undercount its documented model;
/// - be deterministic, side-effect-free, and cheap;
/// - not consult mutable interior state; and
/// - return the same charge while the value is logically unchanged.
///
/// Shared backing storage may be conservatively charged once per retaining
/// value. A type whose retained charge might not fit in `u64` should expose a
/// domain-specific fallible measurement instead of implementing this trait.
///
/// For an immutable admitted value, a bounded owner measures exactly once at
/// admission and stores that charge beside the value. It releases the stored
/// charge later; it does not measure the value again during release.
///
/// The stored charge must remain a conservative upper bound for the value's
/// retained storage for its entire lifetime in the bounded owner. Before an
/// admitted value grows, the owner must reserve any additional charge and
/// update the stored charge in the same transaction; otherwise it must prohibit
/// growth.
///
/// [`crate::ByteBudget`] validates aggregate arithmetic, not charge provenance.
/// Releasing an incorrect charge succeeds whenever that count does not exceed
/// aggregate use. The owner must therefore keep the exact charge beside each
/// admitted value and release that charge exactly once.
///
/// Container implementations are intentionally consumer-owned.
///
/// ```compile_fail
/// use bytebudget::Retained;
/// let bytes = vec![1_u8, 2, 3];
/// let _ = bytes.retained_bytes();
/// ```
///
/// Compatibility aliases are deliberately absent.
///
/// ```compile_fail
/// use bytebudget::RetainedBytes;
/// ```
pub trait Retained {
    /// Returns the variable storage retained by this value.
    ///
    /// Ignoring a measurement is rejected when unused results are denied.
    ///
    /// ```compile_fail
    /// #![deny(unused_must_use)]
    /// use bytebudget::Retained;
    /// ().retained_bytes();
    /// ```
    #[must_use = "retained-storage measurements must be admitted or stored"]
    fn retained_bytes(&self) -> ByteCount;
}

impl Retained for () {
    fn retained_bytes(&self) -> ByteCount {
        ByteCount::ZERO
    }
}
