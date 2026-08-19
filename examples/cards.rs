//! Example: deal cards to two players until the deck is nearly exhausted,
//! then answer suit/rank questions and draw by meaning — all bitmask ops.
//!
//! Run with: cargo run --example cards --features=cards

// Suits, ranks, and the joker/color subsets come from the `cards` feature.
use bitdeck::{Card, Color, Rank, Standard, StandardCards, Suit};
use rand::{SeedableRng, rngs::SmallRng};

fn mask_names(mask: u64) -> String {
    Standard::from_bits(mask)
        .iter()
        .map(|id| Card::from_id(id).to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

fn deck_names(deck: Standard) -> String {
    mask_names(deck.as_bits())
}

fn main() {
    let mut rng = SmallRng::seed_from_u64(420);
    let mut deck = Standard::default();

    println!("Full deck has {} cards.", deck.remaining());

    // Deal 5 standard (non-joker) cards to each of two players, round by
    // round, accumulating into a single hand per player. With 52 standard
    // cards this leaves 2 cards unused.
    let hand_size: usize = 5;
    let mut player_a = Standard::empty();
    let mut player_b = Standard::empty();
    let mut round = 1;

    while deck.count_subset(StandardCards) >= 10 {
        let hand_a = deck.draw_subset_mask(&mut rng, StandardCards, hand_size);
        let hand_b = deck.draw_subset_mask(&mut rng, StandardCards, hand_size);
        player_a.insert_all(hand_a);
        player_b.insert_all(hand_b);

        println!("\nRound {round}:");
        round += 1;
        println!(
            "  Drew {} to Player A: {}",
            hand_a.count_ones(),
            mask_names(hand_a)
        );
        println!(
            "  Drew {} to Player B: {}",
            hand_b.count_ones(),
            mask_names(hand_b)
        );
    }

    println!(
        "\nDealt {} rounds. Player A has {} cards, Player B has {} cards. {} cards remain.",
        round,
        player_a.remaining(),
        player_b.remaining(),
        deck.remaining()
    );
    println!("\nPlayer A's hand: {}", deck_names(player_a));
    println!("Player B's hand: {}", deck_names(player_b));

    println!("\nSpades left: {}", deck.count_subset(Suit::Spades));
    println!("Queens left: {}", deck.count_subset(Rank::Queen));

    // Leftover cards (skipping jokers).
    println!("\nRemaining cards:");
    for id in deck.iter_subset(StandardCards) {
        println!("  {:>8} (id {:>2})", Card::from_id(id), id);
    }

    // Restock and remove all black cards in one mask operation.
    deck.restock();
    println!("Restocked.");
    let black = deck.remove_all(Color::Black.mask());
    println!("Drew {black} black cards");
    println!(
        "Black left: {}; Red left: {}",
        deck.count_subset(Color::Black),
        deck.count_subset(Color::Red)
    );
}
