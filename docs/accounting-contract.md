# Accounting contract

`ByteCount` and `ByteBudget` provide fixed-width, checked aggregate accounting.
They do not infer allocator behavior or attach identity to reservations.

## The accounting unit is `u64`

`ByteCount` is stable across ordinary 32-bit and 64-bit targets and independent
of pointer width. Conversions to and from `usize` are fallible whenever the
destination cannot represent the value. A failed conversion returns
`ByteCountOverflow`.

Scalar composition uses only `checked_add` and `checked_sub`. Implicit,
saturating, and wrapping arithmetic are deliberately absent.

## Budget operations are transactional

A budget maintains `used <= limit` beneath an immutable limit.

- Reservation succeeds exactly when the requested amount fits.
- Release succeeds exactly when the requested amount is in use.
- A rejected operation leaves every observable field unchanged.

Capacity rejection returns `CapacityExceeded`; accounting underflow returns
`OverRelease`.

`ByteBudget` is not cloneable, copyable, or defaultable. Reservations have no
identity or guard, so consumers must preserve their own charge provenance and
release each admitted charge exactly once.

Return to the [README](../README.md) or read the
[retained-charge contract](retained-charges.md).
