# Qualification

The canonical local gate is:

```sh
zcheck run check
```

The checked-in graph verifies:

- repository policy, zrail architecture, and clean diffs;
- formatting, Clippy, rustdoc, and `no_std` compilation;
- Rust 1.88 and current-stable tests plus doctests;
- direct, model, persona, and mutation evidence; and
- the packaged archive through a fresh external consumer.

Hosted CI runs the same canonical graph and executes the complete test suite on
a 32-bit target. A passing source-mode build alone is not package or release
proof.

Return to the [README](../README.md).
