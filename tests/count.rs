//! Direct contracts for fixed-width byte counts and integer conversions.

use core::mem::{align_of, size_of};

use bytebudget::{ByteCount, ByteCountOverflow};

const THREE: ByteCount = ByteCount::new(3);
const THREE_RAW: u64 = THREE.get();
const THREE_IS_ZERO: bool = THREE.is_zero();
const FIVE: Option<ByteCount> = THREE.checked_add(ByteCount::new(2));
const ONE: Option<ByteCount> = THREE.checked_sub(ByteCount::new(2));

#[test]
fn transparent_layout_matches_u64() {
    assert_eq!(size_of::<ByteCount>(), size_of::<u64>());
    assert_eq!(align_of::<ByteCount>(), align_of::<u64>());
}

#[test]
fn constants_and_accessors_are_const_usable() {
    assert_eq!(ByteCount::ZERO.get(), 0);
    assert_eq!(ByteCount::MAX.get(), u64::MAX);
    assert_eq!(THREE_RAW, 3);
    assert!(!std::hint::black_box(THREE_IS_ZERO));
    assert_eq!(FIVE, Some(ByteCount::new(5)));
    assert_eq!(ONE, Some(ByteCount::new(1)));
    assert!(ByteCount::ZERO.is_zero());
}

#[test]
fn equality_and_order_follow_the_exact_count() {
    assert_eq!(ByteCount::new(7), ByteCount::new(7));
    assert!(ByteCount::new(6) < ByteCount::new(7));
    assert_eq!(ByteCount::default(), ByteCount::ZERO);
}

#[test]
fn checked_addition_accepts_the_boundary_and_rejects_overflow() {
    assert_eq!(
        ByteCount::new(u64::MAX - 1).checked_add(ByteCount::new(1)),
        Some(ByteCount::MAX)
    );
    assert_eq!(
        ByteCount::MAX.checked_add(ByteCount::ZERO),
        Some(ByteCount::MAX)
    );
    assert_eq!(ByteCount::MAX.checked_add(ByteCount::new(1)), None);
}

#[test]
fn checked_subtraction_accepts_zero_and_rejects_underflow() {
    assert_eq!(
        ByteCount::new(9).checked_sub(ByteCount::new(4)),
        Some(ByteCount::new(5))
    );
    assert_eq!(
        ByteCount::ZERO.checked_sub(ByteCount::ZERO),
        Some(ByteCount::ZERO)
    );
    assert_eq!(ByteCount::ZERO.checked_sub(ByteCount::new(1)), None);
}

#[test]
fn fixed_width_conversions_are_lossless() {
    assert_eq!(ByteCount::from(u32::MAX), ByteCount::new(u64::from(u32::MAX)));
    assert_eq!(ByteCount::from(u64::MAX), ByteCount::MAX);
    assert_eq!(u64::from(ByteCount::MAX), u64::MAX);
}

#[test]
fn usize_conversions_respect_the_pointer_width_boundary() {
    let native = usize::MAX;
    let to_count = ByteCount::try_from(native);
    if usize::BITS <= u64::BITS {
        assert_eq!(to_count, Ok(ByteCount::new(native as u64)));
    } else {
        assert_eq!(to_count, Err(ByteCountOverflow));
    }

    let representable = ByteCount::new(u64::from(u32::MAX));
    assert_eq!(usize::try_from(representable), Ok(u32::MAX as usize));
}

#[cfg(target_pointer_width = "32")]
#[test]
fn usize_rejects_counts_above_the_32_bit_boundary() {
    let too_large = ByteCount::new(u64::from(u32::MAX) + 1);
    assert_eq!(usize::try_from(too_large), Err(ByteCountOverflow));
}

#[cfg(target_pointer_width = "64")]
#[test]
fn usize_accepts_every_byte_count_on_64_bit_targets() {
    assert_eq!(usize::try_from(ByteCount::MAX), Ok(usize::MAX));
}

#[test]
fn conversion_error_is_a_core_error_with_stable_diagnostic_text() {
    fn assert_error<T: core::error::Error>() {}
    assert_error::<ByteCountOverflow>();
    assert_eq!(
        ByteCountOverflow.to_string(),
        "byte count does not fit the target integer type"
    );
}
