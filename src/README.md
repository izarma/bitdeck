## Operation reference

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
