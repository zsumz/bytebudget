# Retained charges

`Retained` reports the variable storage kept alive by one value under its
implementation's documented model. It lets a bounded owner reserve capacity
before accepting that value and release the same charge when ownership ends.

## Measurement happens once

Measure a value at admission, store the charge beside the admitted value, and
release that stored charge later. Do not measure the value again during
release.

Implementations must:

- exclude `size_of::<Self>()` and fixed queue-slot or envelope overhead;
- include transitively owned variable backing storage;
- charge allocated capacity when it remains allocated;
- never undercount; and
- remain deterministic, side-effect-free, and cheap.

They must not consult mutable interior state, and they must return the same
charge while the value is logically unchanged. A type whose retained charge
might not fit in `u64` should expose domain-specific fallible measurement
instead of implementing `Retained`.

Container accounting remains consumer-owned. `bytebudget` does not provide
blanket container implementations.

## Charge lifetime and provenance

A stored charge must remain a conservative upper bound for the value's retained
storage for its entire lifetime in the bounded owner. Keep the exact charge
beside each admitted value and release it exactly once.

`ByteBudget` validates aggregate arithmetic, not charge ownership. Releasing
the wrong charge succeeds whenever that count does not exceed aggregate use.
The bounded owner is responsible for preserving provenance.

## Admitted growth

If an admitted value can grow, reserve the additional charge before installing
that growth and update the stored charge in the same transaction. Otherwise,
prohibit growth while the value remains admitted.

## Shared backing storage

When independently admitted values retain the same backing allocation, each
measurement must remain conservative for its value. Consumers may define a
more precise shared-ownership model, but `ByteBudget` itself records only an
aggregate byte count and cannot authenticate which value owns a charge.

Return to the [README](../README.md).
