//! Example: weighted random pulls — a 9-card loot pool with 5 commons,
//! 3 rares, and 1 legendary, where each pull picks a rarity by weight
//! (5:3:1) and then a uniform card within that rarity.
//!
//! Run with: cargo run --example weighted

use bitdeck::{Deck, meanings};
use rand::{SeedableRng, rngs::SmallRng};

meanings! {
    /// Pull rarity of a card id in the 9-card pool.
    enum Rarity {
        Common, // x 5
        Rare, // x 3
        Legendary, // x 1
    }
    from_id = |id: u8| if id < 5 { 0 } else if id < 8 { 1 } else { 2 };
    cards = 9;
}

/// The 9-card loot pool.
type Pool = Deck<9>;

fn name(id: u8) -> String {
    format!("{:#?} #{id}", Rarity::from_id(id))
}

fn main() {
    let mut rng = SmallRng::seed_from_u64(420);
    println!("\nCracking one full pool (no restock):");
    let mut pool = Pool::default();

    assert_eq!(pool.remaining(), 9);

    // A normal draw naturally weights rarities by how many cards they have.
    while let Some(id) = pool.draw(&mut rng) {
        println!("  Pulled {} ({} drops remain)", name(id), pool.remaining());
    }

    assert!(pool.is_empty());
}
