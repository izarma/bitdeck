# bitdeck

A minimal, fast, deterministic, bitmask-backed shuffle bag for drawing items without replacement, with a built-in implementation of a standard card deck.

[`Deck<N>`] packs the entire state of an `N`-card deck (`N <= 64`) into a single
`u64`. [`StdDeck`] is the usual 52-card deck plus two jokers (`Deck<54>`).

Cards are identified by number eg for [`StdDeck`]:

- `0..=51` : standard cards (`suit = id / 13`, `rank = id % 13`)
- `52, 53` : the two jokers

## Features

- `serde`: serialization support for [`Deck<N>`] / [`StdDeck`].
- `bevy`: derives [`bevy_ecs::prelude::Resource`](https://docs.rs/bevy/latest/bevy/prelude/trait.Resource.html) for [`Deck<N>`] / [`StdDeck`].

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
and the deck answers subset questions directly with [`contains_any`],
[`contains_all`], [`count_in`], and [`draw_in`].

Define regular masks in const with [`stride_mask`], or let [`meanings!`]
generate the enum *and* every variant's mask from a single id mapping.
