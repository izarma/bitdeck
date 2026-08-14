# bitdeck

bitdeck is a fixed-capacity (N ≤ 64) **Bitmask**. `Deck<N>` stores subset membership as a single `u64`, enabling O(1) deterministic subset queries, bulk mutations, and (with the `rand` feature) uniform random draws without replacement and non-destructive peeks.

While it includes a standard 54-card deck preset (`StdDeck`) behind the `cards` feature, `Deck<N>` is completely generic and can be used for loot tables, card decks, turn based action queues, gacha pools etc.

## Features

- `rand` (default): enables the random draw/peek APIs — `Deck::draw`, `Deck::draw_in`, `Deck::draw_into`, `Deck::peek`, and `Deck::peek_in`.
- `serde`: serialization support for [`Deck<N>`].
- `bevy`: derives [`bevy_ecs::prelude::Resource`](https://docs.rs/bevy/latest/bevy/prelude/trait.Resource.html) for [`Deck<N>`].
- `cards`: exposes the `cards` module with `StdDeck` alongside its meaning subsets - standard-card suits, ranks, colors, and predefined masks.

## Properties

- **Uniform without replacement.** Every remaining item is equally likely on
  every draw; drawn items leave the deck.
- **Subset-aware.** Draw from or query any subset — "a heart", "a red card",
  "a common drop" — with a plain `u64` mask.
- **Const mask algebra.** Build masks in const contexts with [`stride_mask`]
  or the [`meanings!`] macro (variant indices follow declaration order);
  compose them with `|`, `&`, and `!`.
- **Bring your own RNG.** All randomness comes from a caller-supplied `rand`
  RNG; the deck itself holds no RNG state.

## Bitmask operations

`Deck<N>` is just a `u64` bitmask. Every operation below is a thin wrapper
around a bitwise read or mutation on that mask, which is why subset queries,
bulk updates, and random draws are all O(1).

<!-- bitdeck-op-table-start -->

| Operation | Mask effect | Description | Feature | Notes |
|-----------|-------------|-------------|---------|-------|
| [`full_mask`] | returns `(1u64 << N) - 1` | Bitmask with the lowest `N` bits set. | | Panics if `N > 64`. |
| [`Default::default`] | `mask = full_mask::<N>()` | Constructor: a full deck. | | Compile-time error if `N > 64`. |
| [`Deck::empty`] | `mask = 0` | Constructor: an empty deck. | | Compile-time error if `N > 64`. |
| [`Deck::from_bits`] | `mask = m & full_mask::<N>()` | Wrap a raw bitmask; high bits are masked off. | | Round-trips with `as_bits`; panics in debug if high bits are set. |
| [`Deck::as_bits`] | returns `mask` | Get the raw remaining bitmask. | | Round-trips with `from_bits`. |
| [`Deck::restock`] | `mask = full_mask::<N>()` | Restore the deck to full. | | |
| [`Deck::insert_all`] | `mask = mask \| (m & full_mask::<N>())` | Add every selected card back. | | Ignores bits at or above `N`. |
| [`Deck::remove_all`] | `mask &= !m` | Remove every selected card; return count removed. | | Ignores bits at or above `N`. |
| [`Deck::retain`] | `mask &= m` | Keep only the selected remaining cards. | | Ignores bits at or above `N`. |
| [`Deck::remaining`] | `mask.count_ones()` | Count cards remaining. | | |
| [`Deck::is_empty`] | `mask == 0` | `true` if no cards remain. | | |
| [`Deck::drawn_mask`] | `!mask & full_mask::<N>()` | Bitmask of cards that have been drawn. | | |
| [`Deck::drawn_count`] | `N - mask.count_ones()` | Count cards that have been drawn. | | |
| [`Deck::iter_drawn`] | iterates `!mask & full_mask::<N>()` | Iterate drawn card ids in ascending order. | | |
| [`Deck::iter_in`] | iterates `mask & m` | Iterate remaining ids inside a subset. | | |
| [`Deck::iter`] | iterates `mask` | Iterate all remaining ids in ascending order. | | |
| [`Deck::nth`] | `select_nth_set(mask, k)` | `k`th remaining card by id. | | Returns `None` if `k >= remaining`. |
| [`Deck::first`] | `mask.isolate_lowest_one()` | Lowest-id remaining card. | | Returns `None` if empty. |
| [`Deck::last`] | `mask.ilog2()` | Highest-id remaining card. | | Returns `None` if empty. |
| [`Deck::peek`] | reads `mask` | Random remaining card without removing it. | `rand` | Returns `None` if empty. |
| [`Deck::peek_in`] | reads `mask & m` | Random remaining card from subset, no removal. | `rand` | Returns `None` if no selected card remains. |
| [`Deck::draw`] | clears one random set bit | Randomly draw and remove one card. | `rand` | Returns `None` if empty. |
| [`Deck::draw_in`] | clears one random bit from `mask & m` | Randomly draw from a subset. | `rand` | Returns `None` if no selected card remains. |
| [`Deck::draw_into`] | repeated [`Deck::draw`] | Draw up to `count` cards into a buffer. | `rand` | Clears `out` first; stops early if deck runs out. |
| [`Deck::contains`] | `(mask >> c) & 1 == 1` | `true` if card `c` is still present. | | Panics if `card >= N`. |
| [`Deck::contains_any`] | `mask & m != 0` | `true` if any selected card remains. | | |
| [`Deck::contains_all`] | `mask & m == m & full_mask::<N>()` | `true` if every selected card remains. | | Ignores bits at or above `N`. |
| [`Deck::count_in`] | `(mask & m).count_ones()` | Count remaining cards in a subset. | | |
| [`Deck::chance`] | `count_in(m) / mask.count_ones()` | Probability a random remaining card is in subset. | | Returns `NaN` when empty. |
| [`Deck::insert`] | `mask = mask \| (1 << c)` | Put one card back. | | Panics if `card >= N`. |
| [`Deck::remove`] | `mask &= !(1 << c)` | Remove one card; return whether it was present. | | Panics if `card >= N`. |

<!-- bitdeck-op-table-end -->
