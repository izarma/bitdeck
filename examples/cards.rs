//! Example: deal cards to two players until the deck is nearly exhausted.
//!
//! Run with: cargo run --example cards

use bitdeck::StdDeck;
use rand::{SeedableRng, rngs::SmallRng};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Suit {
    Clubs,
    Diamonds,
    Hearts,
    Spades,
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

    fn from_id(id: u8) -> Self {
        match id / 13 {
            0 => Suit::Clubs,
            1 => Suit::Diamonds,
            2 => Suit::Hearts,
            3 => Suit::Spades,
            _ => panic!("invalid suit for card id {id}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

impl Rank {
    fn name(self) -> &'static str {
        match self {
            Rank::Ace => "A",
            Rank::Two => "2",
            Rank::Three => "3",
            Rank::Four => "4",
            Rank::Five => "5",
            Rank::Six => "6",
            Rank::Seven => "7",
            Rank::Eight => "8",
            Rank::Nine => "9",
            Rank::Ten => "10",
            Rank::Jack => "J",
            Rank::Queen => "Q",
            Rank::King => "K",
        }
    }

    fn from_id(id: u8) -> Self {
        match id % 13 {
            0 => Rank::Ace,
            1 => Rank::Two,
            2 => Rank::Three,
            3 => Rank::Four,
            4 => Rank::Five,
            5 => Rank::Six,
            6 => Rank::Seven,
            7 => Rank::Eight,
            8 => Rank::Nine,
            9 => Rank::Ten,
            10 => Rank::Jack,
            11 => Rank::Queen,
            12 => Rank::King,
            _ => panic!("invalid rank for card id {id}"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Card {
    Standard(Suit, Rank),
    Joker(u8),
}

impl Card {
    fn from_id(id: u8) -> Self {
        match id {
            52 => Card::Joker(0),
            53 => Card::Joker(1),
            _ => Card::Standard(Suit::from_id(id), Rank::from_id(id)),
        }
    }

    fn name(self) -> String {
        match self {
            Card::Standard(suit, rank) => format!("{}{}", rank.name(), suit.symbol()),
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
    let mut rng = SmallRng::seed_from_u64(42);
    let mut deck = StdDeck::default();

    println!("Full deck has {} cards.", deck.remaining());

    // Deal 5 cards to each of two players, round by round, until the deck
    // cannot satisfy both hands. With 54 cards this leaves 4 cards unused.
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

    // Leftover cards.
    let remaining: Vec<u8> = deck.iter().collect();
    println!("\nRemaining cards:");
    for id in &remaining {
        println!("  {:>8} (id {:>2})", Card::from_id(*id).name(), id);
    }

    // A fresh restock.
    deck.restock();
    println!("After restock: {} cards.", deck.remaining());
}
