# bitdeck

A minimal, fast, deterministic bitmask deck of cards.

[`Deck<N>`] packs the entire state of an `N`-card deck (`N <= 64`) into a single
`u64`. [`StdDeck`] is the usual 52-card deck plus two jokers (`Deck<54>`).

Cards are identified by number eg for [`StdDeck`]:

- `0..=51` : standard cards (`suit = id / 13`, `rank = id % 13`)
- `52, 53` : the two jokers

## Features

- `serde`: serialization support for [`Deck<N>`] / [`StdDeck`].
- `bevy`: derives [`bevy_ecs::prelude::Resource`](https://docs.rs/bevy_ecs/latest/bevy_ecs/prelude/struct.Resource.html) for [`Deck<N>`] / [`StdDeck`].

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
