//! Direct boundary and transactional-state contracts for byte budgets.

use bytebudget::{ByteBudget, ByteCount, CapacityExceeded, OverRelease};

fn count(bytes: u64) -> ByteCount {
    ByteCount::new(bytes)
}

fn state(budget: &ByteBudget) -> (ByteCount, ByteCount, ByteCount, bool) {
    (budget.limit(), budget.used(), budget.available(), budget.is_exhausted())
}

#[test]
fn zero_limit_accepts_zero_and_is_exhausted() {
    let mut budget = ByteBudget::new(ByteCount::ZERO);
    assert_eq!(budget.limit(), ByteCount::ZERO);
    assert_eq!(budget.used(), ByteCount::ZERO);
    assert_eq!(budget.available(), ByteCount::ZERO);
    assert!(budget.can_reserve(ByteCount::ZERO));
    assert!(budget.is_exhausted());
    assert_eq!(budget.try_reserve(ByteCount::ZERO), Ok(()));
    assert_eq!(budget.release(ByteCount::ZERO), Ok(()));
}

#[test]
fn exact_limit_reservation_succeeds_and_exhausts() {
    let mut budget = ByteBudget::new(count(8));
    assert!(budget.can_reserve(count(8)));
    assert_eq!(budget.try_reserve(count(8)), Ok(()));
    assert_eq!(budget.used(), count(8));
    assert_eq!(budget.available(), ByteCount::ZERO);
    assert!(budget.is_exhausted());
}

#[test]
fn repeated_reservations_reach_the_exact_limit() {
    let mut budget = ByteBudget::new(count(10));
    budget.try_reserve(count(3)).unwrap();
    assert_eq!(budget.used(), count(3));
    assert_eq!(budget.available(), count(7));
    budget.try_reserve(count(7)).unwrap();
    assert_eq!(budget.used(), count(10));
    assert_eq!(budget.available(), ByteCount::ZERO);
}

#[test]
fn one_byte_over_is_rejected_with_exact_payload_and_no_mutation() {
    let mut budget = ByteBudget::new(count(10));
    budget.try_reserve(count(4)).unwrap();
    let before = state(&budget);

    let error = budget.try_reserve(count(7)).unwrap_err();

    assert_eq!(error.requested(), count(7));
    assert_eq!(error.available(), count(6));
    assert_eq!(state(&budget), before);
    assert_eq!(
        error.to_string(),
        "requested 7 bytes, but only 6 bytes are available"
    );
}

#[test]
fn zero_reservation_and_release_leave_nonzero_state_unchanged() {
    let mut budget = ByteBudget::new(count(10));
    budget.try_reserve(count(4)).unwrap();
    let before = state(&budget);
    assert_eq!(budget.try_reserve(ByteCount::ZERO), Ok(()));
    assert_eq!(state(&budget), before);
    assert_eq!(budget.release(ByteCount::ZERO), Ok(()));
    assert_eq!(state(&budget), before);
}

#[test]
fn partial_then_exact_release_returns_to_zero() {
    let mut budget = ByteBudget::new(count(10));
    budget.try_reserve(count(10)).unwrap();
    budget.release(count(3)).unwrap();
    assert_eq!(budget.used(), count(7));
    assert_eq!(budget.available(), count(3));
    budget.release(count(7)).unwrap();
    assert_eq!(budget.used(), ByteCount::ZERO);
    assert_eq!(budget.available(), count(10));
    assert!(!budget.is_exhausted());
}

#[test]
fn over_release_is_rejected_with_exact_payload_and_no_mutation() {
    let mut budget = ByteBudget::new(count(10));
    budget.try_reserve(count(4)).unwrap();
    let before = state(&budget);

    let error = budget.release(count(5)).unwrap_err();

    assert_eq!(error.released(), count(5));
    assert_eq!(error.used(), count(4));
    assert_eq!(state(&budget), before);
    assert_eq!(
        error.to_string(),
        "cannot release 5 bytes when 4 bytes are used"
    );
}

#[test]
fn maximum_limit_has_no_hidden_overflow_branch() {
    let mut budget = ByteBudget::new(ByteCount::MAX);
    assert_eq!(budget.try_reserve(ByteCount::MAX), Ok(()));
    assert_eq!(budget.used(), ByteCount::MAX);
    let error = budget.try_reserve(count(1)).unwrap_err();
    assert_eq!(error.requested(), count(1));
    assert_eq!(error.available(), ByteCount::ZERO);
    assert_eq!(budget.release(ByteCount::MAX), Ok(()));
}

#[test]
fn errors_implement_core_error() {
    fn assert_error<T: core::error::Error>() {}
    assert_error::<CapacityExceeded>();
    assert_error::<OverRelease>();
}
