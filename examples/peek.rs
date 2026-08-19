//! Example: peek at cards, then decide what stays on top — the "scry"
//! pattern. The bitmask deck has no order, so "on top" is modeled as a small
//! ordered queue *next to* the deck: staged and kept cards leave the random
//! pool, and "shuffle back" is a single `insert` — a uniform bag is always
//! shuffled.
//!
//! Run with: cargo run --example peek --features=cards

use bitdeck::{Standard, cards::card_name};
use rand::{SeedableRng, rngs::SmallRng};
use std::collections::VecDeque;

/// A bitmask deck plus the one thing a bitmask cannot hold: order.
///
/// - `top` is the known, ordered zone — cards someone peeked and chose to
///   keep on top. `top[0]` is drawn next.
/// - `staging` holds cards currently being peeked, awaiting the player's
///   keep-or-shuffle decision.
/// - Everything else lives in the bitmask: unordered, uniform random.
struct ScryDeck {
    deck: Standard,
    top: VecDeque<u8>,
    staging: Vec<u8>,
}

impl ScryDeck {
    fn new() -> Self {
        Self {
            deck: Standard::default(),
            top: VecDeque::new(),
            staging: Vec::new(),
        }
    }

    /// Moves up to `k` cards out of the random pool into the staging area
    /// and returns them for the player to look at.
    ///
    /// Staged cards are *out of the deck*: they cannot be drawn by anyone
    /// until [`resolve`](Self::resolve) puts them back or on top.
    fn stage_many(&mut self, rng: &mut impl rand::Rng, k: usize) -> &[u8] {
        debug_assert!(
            self.staging.is_empty(),
            "resolve the previous peek before peeking again"
        );
        self.deck.draw_into(rng, k, &mut self.staging);
        &self.staging
    }

    /// Resolves a peek: the staged cards at `keep_in_draw_order` stay on top
    /// (first index is drawn first); every other staged card shuffles back.
    fn resolve(&mut self, keep_in_draw_order: &[usize]) {
        for &i in keep_in_draw_order {
            self.top.push_back(self.staging[i]);
        }
        for (i, &card) in self.staging.iter().enumerate() {
            // `keep_in_draw_order` is tiny for a typical scry, so the O(n²)
            // `contains` is simpler than building a set here.
            if !keep_in_draw_order.contains(&i) {
                self.deck.insert(card); // "shuffle back" = set the bit again
            }
        }
        self.staging.clear();
    }
}

fn main() {
    let mut rng = SmallRng::seed_from_u64(420);
    let mut scry = ScryDeck::new();

    println!("Full deck: {} cards.\n", scry.deck.remaining());

    let peeked = scry.stage_many(&mut rng, 3).to_vec();
    println!("Scry 3 — peeked:");
    for (i, &id) in peeked.iter().enumerate() {
        println!("  [{i}] {}", card_name(id));
    }

    // Keep [0] and [2] in draw order; [1] returns to the random pool.
    let (first_kept, second_kept, returned) = (peeked[0], peeked[2], peeked[1]);
    scry.resolve(&[0, 2]);

    println!(
        "\nChose: keep {} on top, then {}; shuffle {} back.",
        card_name(first_kept),
        card_name(second_kept),
        card_name(returned)
    );
    println!(
        "On top: {}, then {}.",
        card_name(first_kept),
        card_name(second_kept)
    );
    println!(
        "Random pool: {} cards (including {}).",
        scry.deck.remaining(),
        card_name(returned)
    );

    assert_eq!(scry.deck.remaining(), 52);
    assert!(scry.deck.contains(returned));
}
