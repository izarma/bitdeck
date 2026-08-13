# bitdeck

An 8-byte shuffle bag: uniform random sampling without replacement for items <=64, with bitmask subset queries and draws. Includes a standard 54-card deck preset.

[`Deck<N>`] stores which of `N` items (`N <= 64`) remain as a single `u64`.
As a preset we have [`StdDeck`], the usual 52-card deck plus two jokers (`Deck<54>`).

Cards are identified by number eg for [`StdDeck`]:
- `0..=51` : standard cards (`suit = id / 13`, `rank = id % 13`)
- `52, 53` : the two jokers

## Features

- `serde`: serialization support for [`Deck<N>`] / [`StdDeck`].
- `bevy`: derives [`bevy_ecs::prelude::Resource`](https://docs.rs/bevy/latest/bevy/prelude/trait.Resource.html) for [`Deck<N>`] / [`StdDeck`].
- `cards`: exposes the [`cards`] module with standard-card suits, ranks, colors, and predefined masks.

## Properties

- **Uniform without replacement.** Every remaining item is equally likely on
  every draw; drawn items leave the deck.
- **Subset-aware.** Draw from or query any subset — "a heart", "a red card",
  "a common drop" — with a plain `u64` mask.
- **Const mask algebra.** Build masks in const contexts with [`stride_mask`]
  or the [`meanings!`] macro; compose them with `|`, `&`, and `!`.
- **Bring your own RNG.** All randomness comes from a caller-supplied `rand`
  RNG; the deck itself holds no RNG state.

## Usage

```rust
use bitdeck::StdDeck;
use rand::{SeedableRng, rngs::SmallRng};

let mut rng = SmallRng::seed_from_u64(42);
let mut deck = StdDeck::default();

// Draw one random card, removing it from the deck.
let card = deck.draw(&mut rng).unwrap();
println!("drew card {card}; {} cards remain", deck.remaining());
```

The RNG-taking APIs accept `&mut impl Rng` from [`rand`] 0.10, so any source
implementing `rand::Rng` plugs in directly.

## Meanings: suits and ranks as bitmasks

Questions about a *set* of cards — "any kings left?", "how many hearts?" —
never need per-card mapping code. A meaning is just a `u64` mask of card ids,
and the deck answers subset questions directly with [`Deck::contains_any`],
[`Deck::contains_all`], [`Deck::count_in`], and [`Deck::draw_in`].

Define regular masks in const with [`stride_mask`], or let [`meanings!`]
generate the enum *and* every variant's mask from a single id mapping, that is inverted at compile time, so suits, ranks, colors, or any custom
classification cost nothing at runtime. See `examples/cards.rs`
(`cargo run --example cards`) for a full game-style walkthrough.
