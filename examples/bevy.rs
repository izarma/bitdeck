//! Bevy integration example.
//!
//! Run with:
//!
//! ```sh
//! cargo run --example bevy --features "bevy cards rand"
//! ```

use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bitdeck::{Card, Standard, StandardCards};
use rand::{SeedableRng, rngs::SmallRng};

/// Marker for entities that carry their own deck.
#[derive(Component)]
struct Player;

/// A single global deck, wrapped as a Bevy `Resource`.
#[derive(Resource)]
struct GameDeck(Standard);

fn main() {
    App::new()
        .add_systems(
            Startup,
            (setup_shared_deck, spawn_players, draw_from_game_deck).chain(),
        )
        .run();
}

fn setup_shared_deck(mut commands: Commands) {
    commands.insert_resource(GameDeck(Standard::default()));
}

fn spawn_players(mut commands: Commands) {
    // Each player gets their own deck component as their hands.
    commands.spawn((Player, Standard::empty()));
    commands.spawn((Player, Standard::empty()));
}

fn draw_from_game_deck(
    mut game_deck: ResMut<GameDeck>,
    mut players: Query<&mut Standard, With<Player>>,
) {
    let mut rng = SmallRng::seed_from_u64(420);

    for (i, mut hand) in players.iter_mut().enumerate() {
        // Draw 5 random non-joker cards from the shared deck into this player's hand.
        let drawn = game_deck.0.draw_subset_mask(&mut rng, StandardCards, 5);
        hand.insert_all(drawn);

        println!("Player {} drew:", i + 1);
        for id in hand.iter() {
            println!("  {}", Card::from_id(id));
        }
    }
}
