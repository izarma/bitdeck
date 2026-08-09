# bitdeck

A minimal, fast, deterministic bitmask deck of cards.

## Features

- `serde`: serialization support for [`StdDeck`].
- `bevy`: derives [`bevy_ecs::prelude::Resource`] for [`StdDeck`].

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
