# bitdeck

bitdeck is a fixed-capacity (N ≤ 64) **Bitset**. `Deck<N>` stores subset membership as a single `u64`, enabling O(1) deterministic subset queries, bulk mutations, and (with the `rand` feature) uniform random sampling without replacement.

While it includes a standard 54-card deck preset (`StdDeck`) behind the `cards` feature, `Deck<N>` is completely generic and can be used for loot tables, card decks, turn based action queues, gacha pools etc.

## Features

- `rand` (default): enables the random draw/peek APIs — `Deck::draw`, `Deck::draw_in`, `Deck::draw_into`, `Deck::peek`, and `Deck::peek_in`.
- `serde`: serialization support for [`Deck<N>`] / `StdDeck`.
- `bevy`: derives [`bevy_ecs::prelude::Resource`](https://docs.rs/bevy/latest/bevy/prelude/trait.Resource.html) for [`Deck<N>`] / `StdDeck`.
- `cards`: exposes the `cards` module with standard-card suits, ranks, colors, and predefined masks.

## Properties

- **Uniform without replacement.** Every remaining item is equally likely on
  every draw; drawn items leave the deck.
- **Subset-aware.** Draw from or query any subset — "a heart", "a red card",
  "a common drop" — with a plain `u64` mask.
- **Const mask algebra.** Build masks in const contexts with [`stride_mask`]
  or the [`meanings!`] macro; compose them with `|`, `&`, and `!`.
- **Bring your own RNG.** All randomness comes from a caller-supplied `rand`
  RNG; the deck itself holds no RNG state.
