# bitdeck

A minimal, fast, deterministic bitmask deck of cards.

[`StdDeck`] packs the entire state of a 54-card deck (52 standard cards plus two
jokers) into a single `u64`.

Cards are identified by number:

- `0..=51` : standard cards (`suit = id / 13`, `rank = id % 13`)
- `52, 53` : the two jokers

## Features

- `serde`: serialization support for [`StdDeck`].
- `bevy`: derives [`bevy_ecs::prelude::Resource`](https://docs.rs/bevy_ecs/latest/bevy_ecs/prelude/struct.Resource.html) for [`StdDeck`].

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
implementing the low-level [`Rng`](rand::Rng) trait plugs in directly.
