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
/// A bounded owner measures exactly once at admission and stores that charge
/// beside the admitted value. It releases the stored admission charge later;
/// it does not measure the value again during release.
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
    fn retained_bytes(&self) -> ByteCount;
}

impl Retained for () {
    fn retained_bytes(&self) -> ByteCount {
        ByteCount::ZERO
    }
}
