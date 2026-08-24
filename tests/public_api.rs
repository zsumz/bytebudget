//! Compile-time and runtime contracts for the intentionally small root API.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use bytebudget::{
    ByteBudget, ByteCount, ByteCountOverflow, CapacityExceeded, OverRelease,
    Retained,
};

const EMPTY: ByteBudget = ByteBudget::new(ByteCount::new(9));
const LIMIT: ByteCount = EMPTY.limit();
const USED: ByteCount = EMPTY.used();
const AVAILABLE: ByteCount = EMPTY.available();
const CAN_RESERVE: bool = EMPTY.can_reserve(ByteCount::new(9));
const EXHAUSTED: bool = EMPTY.is_exhausted();

#[test]
fn the_six_root_types_are_directly_usable() {
    fn assert_error<T: core::error::Error>() {}
    fn assert_copy<T: Copy>() {}

    assert_error::<ByteCountOverflow>();
    assert_error::<CapacityExceeded>();
    assert_error::<OverRelease>();
    assert_copy::<ByteCount>();
    assert_copy::<ByteCountOverflow>();
    assert_copy::<CapacityExceeded>();
    assert_copy::<OverRelease>();
    assert_eq!(().retained_bytes(), ByteCount::ZERO);
}

#[test]
fn public_method_signatures_are_pinned() {
    let _: fn(ByteCount) -> ByteBudget = ByteBudget::new;
    let _: fn(&ByteBudget) -> ByteCount = ByteBudget::limit;
    let _: fn(&ByteBudget) -> ByteCount = ByteBudget::used;
    let _: fn(&ByteBudget) -> ByteCount = ByteBudget::available;
    let _: fn(&ByteBudget, ByteCount) -> bool = ByteBudget::can_reserve;
    let _: fn(&ByteBudget) -> bool = ByteBudget::is_exhausted;
    let _: fn(&mut ByteBudget, ByteCount) -> Result<(), CapacityExceeded> =
        ByteBudget::try_reserve;
    let _: fn(&mut ByteBudget, ByteCount) -> Result<(), OverRelease> =
        ByteBudget::release;
    let _: fn(&CapacityExceeded) -> ByteCount = CapacityExceeded::requested;
    let _: fn(&CapacityExceeded) -> ByteCount = CapacityExceeded::available;
    let _: fn(&OverRelease) -> ByteCount = OverRelease::released;
    let _: fn(&OverRelease) -> ByteCount = OverRelease::used;
    let _: fn(&()) -> ByteCount = <() as Retained>::retained_bytes;
}

#[test]
fn budget_queries_and_scalar_operations_are_const_usable() {
    assert_eq!(LIMIT, ByteCount::new(9));
    assert_eq!(USED, ByteCount::ZERO);
    assert_eq!(AVAILABLE, ByteCount::new(9));
    assert!(std::hint::black_box(CAN_RESERVE));
    assert!(!std::hint::black_box(EXHAUSTED));
}

#[test]
fn byte_count_supports_hashing_without_losing_identity() {
    let mut first = DefaultHasher::new();
    ByteCount::new(37).hash(&mut first);
    let mut second = DefaultHasher::new();
    ByteCount::new(37).hash(&mut second);
    assert_eq!(first.finish(), second.finish());
}
