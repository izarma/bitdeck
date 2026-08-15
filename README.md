# bitdeck

bitdeck is a fixed-capacity (N ≤ 64) **Bitmask**. [`Deck<N>`] stores subset membership as a `u64`, enabling deterministic subset queries, bulk mutations, and (with the `rand` feature) uniform random draws without replacement and non-destructive peeks.

While it includes a standard 54-card deck preset (`StdDeck`) behind the `cards` feature, [`Deck<N>`] is completely generic and can be used for loot tables, turn based action queues, shuffle bags etc.

## Features

- `rand` (default): enables the random draw/peek APIs, including the `*_mask` bulk helpers.
- `alloc` (default): enables the draw/peek `*_into`  helpers that fill an `alloc::vec::Vec`. Requires `rand`.
- `serde`: transparent `u64` bitmask serialization for `Deck<N>`.
- `bevy`: derives [`Component`](https://docs.rs/bevy_ecs/latest/bevy_ecs/component/index.html) and [`Reflect`](https://docs.rs/bevy_reflect/latest/bevy_reflect/trait.Reflect.html) for [`Deck<N>`], so it can be attached to entities, or wrapped in a [`bevy_ecs::prelude::Resource`](https://docs.rs/bevy/latest/bevy/prelude/trait.Resource.html).
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

<!-- bitdeck-op-table-start -->

Bits at or above `N` in any subset mask are ignored. The exception is [`Deck::from_bits`], which also panics in debug builds if out-of-range bits are set.

#### Constructors & conversion

| Operation | Mask effect | Description | Notes |
|-----------|-------------|-------------|-------|
| [`Default::default`] | `mask = full_mask::<N>()` | Constructor: a full deck. | Compile-time error if `N > 64`. |
| [`full_mask`] | returns `(1u64 << N) - 1` (`u64::MAX` when `N = 64`) | ready-made bitmask for **all `N` cards**. | Panics if `N > 64`. |
| [`Deck::empty`] | `mask = 0` | Constructor: an empty deck. | Compile-time error if `N > 64`. |
| [`Deck::from_bits`] | `mask = m & full_mask::<N>()` | Creates a deck from a raw/saved bitmask. | Round-trips with `as_bits`; panics in debug if high bits are set. |
| [`Deck::as_bits`] | returns `mask` | Get the raw remaining bitmask of the deck. | Round-trips with `from_bits`. |

#### Mutations

| Operation | Mask effect | Description | Notes |
|-----------|-------------|-------------|-------|
| [`Deck::restock`] | `mask = full_mask::<N>()` | Restore the deck to full. | |
| [`Deck::insert_all`] | `mask = mask \| (m & full_mask::<N>())` | Add every selected card back. | |
| [`Deck::remove_all`] | `mask &= !m` | Remove every selected remaining card; return count removed. | |
| [`Deck::retain`] | `mask &= m` | Keep only the selected remaining cards. | |
| [`Deck::insert`] | `mask = mask \| (1 << c)` | Put one card back. | Panics if `card >= N`; no effect if already present. |
| [`Deck::remove`] | `mask &= !(1 << c)` | Remove one card; return whether it was present. | Panics if `card >= N`. |
| [`Deck::toggle`] | `mask ^= (1 << c)` | Flip one card’s presence; return new state. | Panics if `card >= N`. |
| [`Deck::toggle_all`] | `mask ^= m & full_mask::<N>()` | Flip every selected card’s presence. | High bits are masked off. |

#### Cardinality & drawn state

| Operation | Mask effect | Description |
|-----------|-------------|-------------|
| [`Deck::remaining`] | `mask.count_ones()` | Count remaining cards. |
| [`Deck::is_empty`] | `mask == 0` | `true` if no cards remain. |
| [`Deck::is_full`] | `mask == full_mask::<N>()` | `true` if every card remains. |
| [`Deck::drawn_mask`] | `!mask & full_mask::<N>()` | Bitmask of cards that have been drawn. |
| [`Deck::drawn_count`] | `N - mask.count_ones()` | Count cards that have been drawn. |

#### Iteration

| Operation | Mask effect | Description |
|-----------|-------------|-------------|
| [`Deck::iter_drawn`] | iterates `!mask & full_mask::<N>()` | Iterate drawn card ids in ascending order. |
| [`Deck::iter_in`] | iterates `mask & m` | Iterate remaining ids inside a subset. |
| [`Deck::iter`] | iterates `mask` | Iterate all remaining ids in ascending order. |

#### Selection

| Operation | Mask effect | Description | Feature | Notes |
|-----------|-------------|-------------|---------|-------|
| [`Deck::nth`] | `select_nth_set(mask, k)` | `k`th remaining card by id. | | Returns `None` if `k >= remaining`. |
| [`Deck::first`] | `mask.isolate_lowest_one()` | Lowest-id remaining card. | | Returns `None` if empty. |
| [`Deck::last`] | `mask.ilog2()` | Highest-id remaining card. | | Returns `None` if empty. |
| [`Deck::nth_in`] | `select_nth_set(mask & m, k)` | `k`th remaining card in subset by id. | | Returns `None` if `k >= count_in(m)`. |
| [`Deck::first_in`] | `select_nth_set(mask & m, 0)` | Lowest-id remaining card in subset. | | Returns `None` if no selected card remains. |
| [`Deck::last_in`] | `select_nth_set(mask & m, count - 1)` | Highest-id remaining card in subset. | | Returns `None` if no selected card remains. |
| [`Deck::draw`] | clears one random set bit | Randomly draw and remove one card. | `rand` | Returns `None` if empty. |
| [`Deck::draw_in`] | clears one random bit from `mask & m` | Randomly draw from a subset. | `rand` | Returns `None` if no selected card remains. |
| [`Deck::draw_mask`] | clears up to `count` random set bits | Randomly draw up to `count` cards. | `rand` | Returns drawn cards as a `u64` bitmask. |
| [`Deck::draw_in_mask`] | clears up to `count` random bits from `mask & m` | Randomly draw up to `count` cards from a subset. | `rand` | Returns drawn cards as a `u64` bitmask. |
| [`Deck::draw_into`] | repeated [`Deck::draw`] | Draw up to `count` cards into a buffer. | `rand`, `alloc` | Clears `out` first; stops early if deck runs out. |
| [`Deck::draw_in_into`] | repeated [`Deck::draw_in`] | Draw up to `count` cards from a subset into a buffer. | `rand`, `alloc` | Clears `out` first; stops early if subset runs out. |
| [`Deck::peek`] | reads `mask` | Random remaining card without removing it. | `rand` | Returns `None` if empty. |
| [`Deck::peek_in`] | reads `mask & m` | Random remaining card from subset, no removal. | `rand` | Returns `None` if no selected card remains. |
| [`Deck::peek_mask`] | reads up to `count` random set bits | Peek up to `count` cards. | `rand` | Returns peeked cards as a `u64` bitmask; deck unchanged. |
| [`Deck::peek_in_mask`] | reads up to `count` random bits from `mask & m` | Peek up to `count` cards from a subset. | `rand` | Returns peeked cards as a `u64` bitmask; deck unchanged. |
| [`Deck::peek_into`] | repeated [`Deck::peek`] | Peek up to `count` cards into a buffer. | `rand`, `alloc` | Clears `out` first; deck unchanged. |
| [`Deck::peek_in_into`] | repeated [`Deck::peek_in`] | Peek up to `count` cards from a subset into a buffer. | `rand`, `alloc` | Clears `out` first; deck unchanged. |

#### Queries

| Operation | Mask effect | Description | Notes |
|-----------|-------------|-------------|-------|
| [`Deck::contains`] | `(mask >> c) & 1 == 1` | `true` if card `c` is still present. | Panics if `card >= N`. |
| [`Deck::contains_any`] | `mask & m != 0` | `true` if any selected card remains. | |
| [`Deck::contains_all`] | `mask & m == m & full_mask::<N>()` | `true` if every selected card remains. | |
| [`Deck::is_subset_of`] | `mask & !m == 0` | `true` if every remaining card is in `m`. | | Empty deck is a subset of every mask. |
| [`Deck::count_in`] | `(mask & m).count_ones()` | Count remaining cards in a subset. | |
| [`Deck::chance`] | `count_in(m) / mask.count_ones()` | Probability a random remaining card is in subset. | Returns `NaN` when empty. |

<!-- bitdeck-op-table-end -->
