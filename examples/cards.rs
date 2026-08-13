//! Example: deal cards to two players until the deck is nearly exhausted,
//! then answer suit/rank questions and draw by meaning — all bitmask ops.
//!
//! Run with: cargo run --example cards

use bitdeck::{StdDeck, meanings, stride_mask};
use rand::{SeedableRng, rngs::SmallRng};

// Primary meanings: Suit and Rank.
// These map every card id 0..52 to exactly one variant.
meanings! {
    /// The four standard suits.
    enum Suit {
        Clubs,
        Diamonds,
        Hearts,
        Spades,
    }
    from_id = |id: u8| id / 13;
    cards = 52;
}

impl Suit {
    fn symbol(self) -> &'static str {
        match self {
            Suit::Clubs => "♣",
            Suit::Diamonds => "♦",
            Suit::Hearts => "♥",
            Suit::Spades => "♠",
        }
    }
}

meanings! {
    /// The 13 ranks.
    enum Rank {
        Ace,
        Two,
        Three,
        Four,
        Five,
        Six,
        Seven,
        Eight,
        Nine,
        Ten,
        Jack,
        Queen,
        King,
    }
    from_id = |id: u8| id % 13;
    cards = 52;
}

// Joker Mask
const JOKER_START: u8 = 52;
const JOKERS: u64 = stride_mask(JOKER_START, 1, 2);

// Composed Color masks.
const RED: u64 = Suit::Diamonds.mask() | Suit::Hearts.mask();
const BLACK: u64 = Suit::Clubs.mask() | Suit::Spades.mask();

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Card {
    Standard(Suit, Rank),
    Joker(u8),
}

impl Card {
    fn from_id(id: u8) -> Self {
        match id {
            id if id >= JOKER_START => Card::Joker(id - JOKER_START),
            _ => Card::Standard(Suit::from_id(id), Rank::from_id(id)),
        }
    }

    fn name(self) -> String {
        match self {
            Card::Standard(suit, rank) => format!("{:#?}{}", rank, suit.symbol()),
            Card::Joker(n) => format!("Joker {n}"),
        }
    }
}

fn hand_names(hand: &[u8]) -> String {
    hand.iter()
        .map(|id| Card::from_id(*id).name())
        .collect::<Vec<_>>()
        .join(", ")
}

fn main() {
    let mut rng = SmallRng::seed_from_u64(420);
    let mut deck = StdDeck::default();

    println!("Full deck has {} cards.", deck.remaining());

    // Remove Jokers before dealing.
    let mut jokers = Vec::new();
    while let Some(id) = deck.draw_in(&mut rng, JOKERS) {
        jokers.push(Card::from_id(id).name());
    }
    println!("\nDrew {} jokers: {}", jokers.len(), jokers.join(", "));

    // Deal 5 cards to each of two players, round by round, until the deck
    // cannot satisfy both hands. With 52 cards this leaves 2 cards unused.
    let hand_size: usize = 5;
    let mut player_a: Vec<Vec<u8>> = Vec::new();
    let mut player_b: Vec<Vec<u8>> = Vec::new();

    while deck.remaining() >= (hand_size * 2) as u32 {
        let mut hand_a = Vec::new();
        let mut hand_b = Vec::new();

        let drawn_a = deck.draw_into(&mut rng, hand_size, &mut hand_a);
        let drawn_b = deck.draw_into(&mut rng, hand_size, &mut hand_b);

        println!("\nRound {}:", player_a.len() + 1);
        println!("  Drew {drawn_a} to Player A: {}", hand_names(&hand_a));
        println!("  Drew {drawn_b} to Player B: {}", hand_names(&hand_b));

        player_a.push(hand_a);
        player_b.push(hand_b);
    }

    println!(
        "\nDealt {} rounds. {} cards remain.",
        player_a.len(),
        deck.remaining()
    );

    println!("\nSpades left: {}", deck.count_in(Suit::Spades.mask()));
    println!("Queens left: {}", deck.count_in(Rank::Queen.mask()));

    // Leftover cards.
    let remaining: Vec<u8> = deck.iter().collect();
    println!("\nRemaining cards:");
    for id in &remaining {
        println!("  {:>8} (id {:>2})", Card::from_id(*id).name(), id);
    }

    // Restock and draw all black cards.
    deck.restock();
    println!("Restocked.");
    let mut black = Vec::new();
    while let Some(id) = deck.draw_in(&mut rng, BLACK) {
        black.push(Card::from_id(id).name());
    }
    println!("Drew {} black cards", black.len());
    println!(
        "Black left: {}; Red left: {}",
        deck.count_in(BLACK),
        deck.count_in(RED)
    );
}
