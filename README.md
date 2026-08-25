<p align="center">
  <img src="bytebudget-logo.svg" alt="bytebudget" width="720">
</p>

<p align="center"><strong>Exact byte accounting for bounded systems.</strong></p>

<p align="center">
  A tiny, deterministic vocabulary for retained-storage charges and bounded
  capacity accounting.
</p>

<p align="center">
  <a href="#install">Install</a>
  <span> · </span>
  <a href="#use">Use</a>
  <span> · </span>
  <a href="#model">Model</a>
  <span> · </span>
  <a href="#measurement-happens-once">Retained storage</a>
  <span> · </span>
  <a href="#version-policy">Version policy</a>
  <span> · </span>
  <a href="#scope">Scope</a>
</p>

<br />

## Install

```toml
[dependencies]
bytebudget = "=0.0.1-rc.1"
```

## Use

```rust
use bytebudget::{ByteBudget, ByteCount};

fn main() {
    let mut budget = ByteBudget::new(ByteCount::new(1_024));
    let charge = ByteCount::new(256);

    budget
        .try_reserve(charge)
        .expect("the budget has enough capacity");
    assert_eq!(budget.available(), ByteCount::new(768));

    budget.release(charge).expect("the charge is in use");
    assert_eq!(budget.used(), ByteCount::ZERO);
}
```

## Model

| Type | Purpose |
| --- | --- |
| `ByteCount` | Fixed-width byte quantity with checked arithmetic |
| `Retained` | Consumer-defined retained-storage measurement |
| `ByteBudget` | Aggregate use beneath an immutable limit |

Conversions use `ByteCountOverflow`, capacity rejection uses
`CapacityExceeded`, and accounting underflow uses `OverRelease`.

## The accounting unit is `u64`

`ByteCount` is stable across ordinary 32-bit and 64-bit targets and independent
of pointer width. `usize` conversions are fallible, while scalar composition
uses only `checked_add` and `checked_sub`; implicit, saturating, and wrapping
arithmetic are deliberately absent.

## Measurement happens once

`Retained` reports the variable storage kept alive by one value under the
implementation's documented model. Implementations include transitively owned
backing storage, charge allocated capacity when it remains allocated, never
undercount, and remain deterministic and side-effect-free.

Measure at admission, store the charge beside the admitted value, and release
that stored charge later. Do not measure the value again during release.
Container accounting remains consumer-owned; there are no blanket container
implementations.

## Charge lifetime and provenance

A stored charge must remain a conservative upper bound for the value's
retained storage for its entire lifetime in the bounded owner. If an admitted
value can grow, reserve the additional charge before installing that growth and
update the stored charge in the same transaction. Otherwise, prohibit growth.

`ByteBudget` validates aggregate arithmetic, not charge ownership. Releasing
the wrong charge succeeds whenever that count does not exceed aggregate use.
Keep the exact charge beside each admitted value and release it exactly once.

## Budget operations are transactional

A budget maintains `used <= limit`. Reservation succeeds exactly when the
requested amount fits; release succeeds exactly when the amount is in use.
Rejected operations leave every observable field unchanged.

`ByteBudget` is not cloneable, copyable, or defaultable. Its limit is immutable,
and reservations have no identity or guard.

## Scope

`bytebudget` is `no_std`, allocation-free, dependency-free, feature-free, and
contains no unsafe code. It does not inspect allocators, parse byte units,
perform I/O, synchronize budgets, track reservation identities, change limits,
or choose eviction and backpressure policy.

## Qualification

```sh
zcheck run check
```

The canonical graph covers formatting, Clippy, Rust 1.88 and stable tests,
doctests, rustdoc, `no_std`, mutation qualification, package verification, and
zrail architecture policy.

## Version policy

The minimum supported Rust version is 1.88 and the qualification graph executes
it directly. MSRV increases are treated as compatibility changes. Before 1.0,
releases may deliberately refine the public API; use an exact version pin when
evaluating a release candidate.

## License

The crate is Apache-2.0 licensed. See [LICENSE](LICENSE). The logo embeds Space
Grotesk under the SIL Open Font License 1.1; see
[LICENSES/Space-Grotesk-OFL.txt](LICENSES/Space-Grotesk-OFL.txt).
