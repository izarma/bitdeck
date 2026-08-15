# bitdeck

bitdeck is a fixed-capacity (N ≤ 64) **Bitmask**. [`Deck<N>`] stores subset membership as a `u64`, enabling deterministic subset queries, bulk mutations, and (with the `rand` feature) uniform random draws without replacement and non-destructive peeks.

While it includes a standard 54-card deck preset (`StdDeck`) behind the `cards` feature, [`Deck<N>`] is completely generic and can be used for loot tables, turn based action queues, shuffle bags etc.

## Features

- `rand` (default): enables the random draw/peek APIs, including the `*_mask` bulk helpers.
- `alloc` (default): enables the draw/peek `*_into`  helpers that fill an `alloc::vec::Vec`. Requires `rand`.
- `serde`: transparent `u64` bitmask serialization for `Deck<N>`.
- `bevy`: derives [`Component`](https://docs.rs/bevy_ecs/latest/bevy_ecs/component/index.html) and [`Reflect`](https://docs.rs/bevy_reflect/latest/bevy_reflect/trait.Reflect.html) for [`Deck<N>`], so it can be attached to entities, or wrapped in a [`Resource`](https://docs.rs/bevy/latest/bevy/prelude/trait.Resource.html).
- `cards`: exposes the `cards` module with `StdDeck` alongside its meaning subsets - standard-card suits, ranks, colors, and predefined masks.

The crate is `no_std`. The default feature set includes `alloc`; disable default features and enable only the features you need for a `no_std` environment without an allocator.

## Properties

- **Uniform without replacement.** Every remaining item is equally likely on every draw; drawn items leave the deck. Use multiple copies of each item for weighted randomness.
- **Subset-aware.** Draw from or query any subset(eg: a heart, a red card, a common drop), with a plain `u64` mask.
- **Const mask algebra.** Build masks in const contexts with [`stride_mask`] or the [`meanings!`] macro (variant indices follow declaration order); compose them with `|`, `&`, and `!`.
- **Bring your own RNG.** All randomness comes from a caller-supplied `rand` RNG; the deck itself holds no RNG state.

## Bitmask operations

[`Deck<N>`] is just a `u64` bitmask. Every operation below is a thin wrapper
around a bitwise read or mutation on that mask.
- Bitwise subset queries and bulk mutations are O(1).
- Random draws/peeks are uniform without replacement:
  O(1) on x86_64 with BMI2, and O(N) with the portable fallback.

Compile with `-C target-feature=+bmi2` to enable the BMI2 fast path at compile
time (no runtime CPUID check):

```bash
RUSTFLAGS="-C target-feature=+bmi2" cargo build
```

Note that this produces a binary that requires BMI2 at runtime; the default
portable build still auto-detects BMI2 via CPUID and uses it when available.
