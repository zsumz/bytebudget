//! Exact byte accounting for bounded systems.
//!
//! `bytebudget` provides three deliberately small concepts:
//!
//! - [`ByteCount`] is a fixed-width byte quantity with checked arithmetic.
//! - [`Retained`] is a consumer-supplied retained-storage contract.
//! - [`ByteBudget`] maintains aggregate use beneath a fixed limit.
//!
//! The crate does not inspect memory or allocators. Owners define what their
//! values retain, measure once when admitting a value, and store that charge
//! beside the value until release.
//!
//! # Accounting model
//!
//! A budget preserves one invariant: `used <= limit`. Reservation succeeds
//! exactly when the requested count is no greater than the available count.
//! Release succeeds exactly when the released count is no greater than the
//! used count. Every rejected operation leaves the budget unchanged.
//!
//! ```
//! use bytebudget::{ByteBudget, ByteCount, Retained};
//!
//! struct Payload {
//!     retained: ByteCount,
//! }
//!
//! impl Retained for Payload {
//!     fn retained_bytes(&self) -> ByteCount {
//!         self.retained
//!     }
//! }
//!
//! struct Entry<T> {
//!     value: T,
//!     charged: ByteCount,
//! }
//!
//! let mut budget = ByteBudget::new(ByteCount::new(64));
//! let value = Payload { retained: ByteCount::new(24) };
//! let charged = value.retained_bytes();
//! budget.try_reserve(charged)?;
//! let entry = Entry { value, charged };
//!
//! // The stored admission charge, not a new measurement, is released.
//! budget.release(entry.charged)?;
//! assert_eq!(budget.used(), ByteCount::ZERO);
//! # let _ = entry.value;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

#![no_std]
#![forbid(unsafe_code)]

mod budget;
mod count;
mod retained;

pub use budget::{ByteBudget, CapacityExceeded, OverRelease};
pub use count::{ByteCount, ByteCountOverflow};
pub use retained::Retained;
