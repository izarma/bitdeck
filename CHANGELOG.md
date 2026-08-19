# Changelog

All notable/breaking changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0]

### Changed

- **Breaking:** replaced the `meanings!` macro and `StdDeck` type with the
  `deck!` macro. Deck newtypes, classification enums, and fixed subsets are now
  generated together and scoped to a specific `Deck<N>`.
- **Breaking:** removed `stride_mask`. Equivalent bitmasks are now produced by
  the generated `mask()` methods and `ALL` constants.

### Migration

If you previously wrote:

```rust
use bitdeck::{meanings, StdDeck};

meanings! {
    pub enum Suit { Clubs, Diamonds, Hearts, Spades }
    from_id = |id: u8| id / 13;
    cards = 52;
}
```

you now write:

```rust
use bitdeck::{Deck, deck};

deck! {
    pub struct Standard = Deck<54>;

    subsets {
        pub enum Suit { Clubs, Diamonds, Hearts, Spades }
        from_id = |id: u8| id / 13;
        cards = 52;
    }
}
```

Use `Suit::Hearts.mask()` (or `Suit::ALL`) wherever you previously used a
`stride_mask` value, and use `Standard::default()` in place of `StdDeck::new()`.

## [0.1.3]

Earlier 0.1.x releases used the `meanings!` macro and `StdDeck` API that was
removed in 0.2.0.
