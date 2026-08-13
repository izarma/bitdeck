#[cfg(test)]
#[cfg(feature = "rand")]
use rand::{SeedableRng, rngs::SmallRng};

use crate::deck::select_nth_set;
use crate::{Deck, full_mask, meanings, stride_mask};

#[cfg(feature = "cards")]
use crate::{CARD_COUNT, FULL_DECK};

#[cfg(feature = "rand")]
fn test_rng() -> SmallRng {
    SmallRng::seed_from_u64(0x1234_5678_9ABC_DEF0)
}

#[test]
#[should_panic(expected = "selected card id exceeds bitmask size")]
fn stride_mask_panics_when_index_exceeds_bitmask() {
    let _ = stride_mask(60, 2, 3); // 60, 62, then 64 > 63
}

#[test]
#[should_panic(expected = "card id 8 out of range")]
fn insert_panics_on_out_of_range_card() {
    Deck::<8>::default().insert(8);
}

#[test]
#[should_panic(expected = "card id 8 out of range")]
fn remove_panics_on_out_of_range_card() {
    Deck::<8>::default().remove(8);
}

#[test]
#[should_panic(expected = "deck size must fit in a u64 bitmask")]
fn full_mask_panics_above_64() {
    let _ = full_mask::<65>();
}

#[test]
#[should_panic(expected = "card id 4 out of range")]
fn contains_panics_on_out_of_range_card() {
    let _ = Deck::<4>::default().contains(4);
}

#[test]
#[cfg_attr(not(debug_assertions), ignore)]
#[should_panic(expected = "bits at or above deck size are not cards")]
fn from_bits_panics_on_bits_above_size_in_debug() {
    // In release builds high bits are silently masked, so the test is ignored.
    let _ = Deck::<4>::from_bits(u64::MAX);
}

#[test]
fn default_remaining_and_full_mask() {
    assert_eq!(Deck::<1>::default().remaining(), 1);
    assert_eq!(Deck::<13>::default().remaining(), 13);
    assert_eq!(Deck::<64>::default().remaining(), 64);
    assert_eq!(Deck::<13>::empty().remaining(), 0);
    assert_eq!(full_mask::<64>(), u64::MAX);
}

#[test]
#[cfg(feature = "cards")]
fn full_deck_matches_card_count() {
    assert_eq!(FULL_DECK, full_mask::<CARD_COUNT>());
}

#[test]
fn as_bits_round_trips_generic() {
    let mut deck = Deck::<10>::default();
    deck.remove(3);
    deck.remove(7);
    let mask = deck.as_bits();
    let restored = Deck::<10>::from_bits(mask);
    assert_eq!(restored, deck);
}

#[test]
fn iter_order_double_ended_and_into_iter() {
    let mut deck = Deck::<5>::default();
    deck.remove(2);

    let mut iter = deck.iter();
    assert_eq!(iter.len(), 4);
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next_back(), Some(4));
    assert_eq!(iter.len(), 2);
    assert_eq!(iter.collect::<Vec<_>>(), vec![1, 3]);

    let collected: Vec<_> = (&deck).into_iter().collect();
    assert_eq!(collected, vec![0, 1, 3, 4]);
}

#[test]
#[cfg(feature = "rand")]
fn draw_all_cards_leaves_deck_empty() {
    let mut deck = Deck::<32>::default();
    let mut rng = test_rng();
    let mut drawn = Vec::with_capacity(32);
    while let Some(card) = deck.draw(&mut rng) {
        drawn.push(card);
    }
    assert!(deck.is_empty());
    drawn.sort_unstable();
    assert_eq!(drawn, (0u8..32).collect::<Vec<_>>());
}

#[test]
#[cfg(feature = "rand")]
fn draw_from_full_64_deck_exercises_top_bit() {
    let mut deck = Deck::<64>::default();
    let mut rng = test_rng();
    let mut drawn = Vec::with_capacity(64);
    while let Some(card) = deck.draw(&mut rng) {
        drawn.push(card);
    }
    assert!(deck.is_empty());
    drawn.sort_unstable();
    assert_eq!(drawn, (0u8..64).collect::<Vec<_>>());
}

#[test]
fn iter_is_fused_after_exhaustion() {
    let mut iter = Deck::<4>::empty().iter();
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);

    let mut iter = Deck::<2>::default().iter();
    assert_eq!(iter.next(), Some(0));
    assert_eq!(iter.next(), Some(1));
    assert_eq!(iter.next(), None);
    // Fused: stays `None`, from both ends.
    assert_eq!(iter.next(), None);
    assert_eq!(iter.next_back(), None);
}

#[cfg(feature = "serde")]
#[test]
fn serde_round_trips_as_transparent_u64() {
    let mut deck = Deck::<54>::default();
    deck.remove(0);
    deck.remove(53);
    let text = ron::to_string(&deck).unwrap();
    // `serde(transparent)`: the deck serializes as its raw mask.
    assert_eq!(text, deck.as_bits().to_string());
    let restored: Deck<54> = ron::from_str(&text).unwrap();
    assert_eq!(restored, deck);
}

#[cfg(feature = "bevy")]
#[test]
fn deck_is_a_usable_bevy_resource() {
    use bevy_ecs::prelude::*;

    fn assert_resource<T: Resource>() {}
    assert_resource::<Deck<54>>();
    assert_resource::<Deck<10>>();

    let mut world = World::new();
    world.insert_resource(Deck::<54>::default());
    assert_eq!(*world.resource::<Deck<54>>(), Deck::<54>::default());
}

#[test]
#[cfg(feature = "rand")]
fn draws_are_deterministic_with_seed() {
    let (mut a, mut b) = (Deck::<16>::default(), Deck::<16>::default());
    let (mut rng_a, mut rng_b) = (test_rng(), test_rng());
    for _ in 0..10 {
        assert_eq!(a.draw(&mut rng_a), b.draw(&mut rng_b));
    }
    for _ in 0..4 {
        assert_eq!(a.draw_in(&mut rng_a, SPREAD), b.draw_in(&mut rng_b, SPREAD));
    }
}

#[test]
#[cfg(feature = "rand")]
fn restock_restores_a_full_deck() {
    let mut deck = Deck::<16>::default();
    let _ = deck.draw_into(&mut test_rng(), 10, &mut Vec::new());
    deck.restock();
    assert_eq!(deck.remaining(), 16);
    assert_eq!(deck, Deck::<16>::default());
}

#[test]
#[cfg(feature = "rand")]
fn draw_into_clears_buffer_and_draws_requested_count() {
    let mut deck = Deck::<16>::default();
    let mut out = vec![255, 254];
    let drawn = deck.draw_into(&mut test_rng(), 5, &mut out);
    assert_eq!(drawn, 5);
    assert_eq!(out.len(), 5);
    assert_eq!(deck.remaining(), 11);
}

#[test]
#[cfg(feature = "rand")]
fn draw_into_stops_when_deck_is_empty() {
    let mut deck = Deck::<16>::default();
    let mut out = Vec::new();
    let drawn = deck.draw_into(&mut test_rng(), 100, &mut out);
    assert_eq!(drawn, 16);
    assert_eq!(out.len(), 16);
    assert!(deck.is_empty());
}

#[test]
fn bulk_mutations_ignore_out_of_range_bits_and_report_removals() {
    let mut deck = Deck::<8>::empty();
    deck.insert_all(0b1_0000_1010);
    assert_eq!(deck.as_bits(), 0b1010);

    assert_eq!(deck.remove_all(0b1_0000_1001), 1);
    assert_eq!(deck.as_bits(), 0b0010);

    deck.insert_all(0b0111_0100);
    deck.retain(0b1_0011_0110);
    assert_eq!(deck.as_bits(), 0b0011_0110);
}

#[test]
fn drawn_card_introspection_and_iteration_are_complementary() {
    let mut deck = Deck::<8>::default();
    assert_eq!(deck.drawn_mask(), 0);
    assert_eq!(deck.drawn_count(), 0);

    deck.remove_all(0b1010_0101);
    assert_eq!(deck.drawn_mask(), 0b1010_0101);
    assert_eq!(deck.drawn_count(), 4);
    assert_eq!(deck.iter_drawn().collect::<Vec<_>>(), vec![0, 2, 5, 7]);
    assert_eq!(deck.iter_in(0b0011_1110).collect::<Vec<_>>(), vec![1, 3, 4]);
}

#[test]
fn deterministic_selection_does_not_mutate_the_deck() {
    let deck = Deck::<8>::from_bits(0b1011_0010);
    assert_eq!(deck.first(), Some(1));
    assert_eq!(deck.last(), Some(7));
    assert_eq!(deck.nth(0), Some(1));
    assert_eq!(deck.nth(1), Some(4));
    assert_eq!(deck.nth(3), Some(7));
    assert_eq!(deck.nth(4), None);
    assert_eq!(deck, Deck::<8>::from_bits(0b1011_0010));

    let empty = Deck::<8>::empty();
    assert_eq!(empty.first(), None);
    assert_eq!(empty.last(), None);
    assert_eq!(empty.nth(0), None);
}

#[test]
#[cfg(feature = "rand")]
fn random_selection_does_not_mutate_the_deck() {
    let deck = Deck::<8>::from_bits(0b1011_0010);
    let mut rng = test_rng();
    let before = deck;
    assert!(matches!(deck.peek(&mut rng), Some(1 | 4 | 5 | 7)));
    assert!(matches!(deck.peek_in(&mut rng, 0b0011_0000), Some(4 | 5)));
    assert_eq!(deck.peek_in(&mut rng, 0b0000_1000), None);
    assert_eq!(deck, before);

    let empty = Deck::<8>::empty();
    assert_eq!(empty.peek(&mut rng), None);
}

#[test]
fn chance_is_the_fraction_of_remaining_cards_in_a_subset() {
    let deck = Deck::<8>::from_bits(0b1011_0010);
    assert_eq!(deck.chance(0b0011_0000), 0.5);
    assert_eq!(deck.chance(0b1_0000_0000), 0.0);
    assert!(Deck::<8>::empty().chance(u64::MAX).is_nan());
}

#[test]
fn insert_contains_remove_roundtrip() {
    let mut deck = Deck::<8>::empty();
    assert!(!deck.contains(7));

    deck.insert(7);
    assert!(deck.contains(7));
    assert_eq!(deck.remaining(), 1);

    // Insert is idempotent.
    deck.insert(7);
    assert_eq!(deck.remaining(), 1);

    assert!(deck.remove(7));
    assert!(!deck.contains(7));
    assert!(deck.is_empty());

    // Removing an absent card reports it.
    assert!(!deck.remove(7));
}

#[test]
fn select_nth_set_selects_kth_set_bit() {
    let mask = 0b1010_1100u64;
    for (k, expected) in [
        (0, 0b0000_0100),
        (1, 0b0000_1000),
        (2, 0b0010_0000),
        (3, 0b1000_0000),
    ] {
        assert_eq!(select_nth_set(mask, k), expected);
    }

    let full = full_mask::<16>();
    for k in [0, 1, 5, 14, 15] {
        assert_eq!(select_nth_set(full, k), 1u64 << k);
    }
}

// meanings

const LOW: u64 = stride_mask(0, 1, 4); // ids 0, 1, 2, 3
const SPREAD: u64 = stride_mask(1, 3, 4); // ids 1, 4, 7, 10
const TAIL: u64 = stride_mask(14, 1, 2); // ids 14, 15

#[test]
fn stride_mask_builds_expected_bits() {
    assert_eq!(LOW, 0b1111);
    assert_eq!(SPREAD, (1 << 1) | (1 << 4) | (1 << 7) | (1 << 10));
    assert_eq!(TAIL, 0b11 << 14);
    // Masks compose with the usual bitwise operators.
    let union = LOW | TAIL;
    assert_eq!(union.count_ones(), 6);
    assert_eq!((LOW & SPREAD).count_ones(), 1); // only id 1
    assert_eq!(full_mask::<14>() | TAIL, full_mask::<16>());
}

#[test]
fn mask_queries_track_subset_membership() {
    let mut deck = Deck::<16>::default();
    assert!(deck.contains_any(LOW));
    assert!(deck.contains_all(SPREAD));
    assert_eq!(deck.count_in(SPREAD), 4);

    // Queries ignore mask bits at or above N.
    assert_eq!(deck.count_in(u64::MAX), 16);

    deck.remove(1);
    assert!(deck.contains_any(SPREAD));
    assert!(!deck.contains_all(SPREAD));
    assert_eq!(deck.count_in(SPREAD), 3);

    for id in 0..4 {
        deck.remove(id);
    }
    assert!(!deck.contains_any(LOW));
    assert_eq!(deck.count_in(LOW), 0);
    // Id 1 was in both subsets, so only 4 distinct cards were removed.
    assert_eq!(deck.remaining(), 16 - 4);
}

#[test]
#[cfg(feature = "rand")]
fn draw_in_draws_only_from_the_subset_until_it_runs_out() {
    let mut deck = Deck::<16>::default();
    let mut rng = test_rng();
    let mut drawn = Vec::new();
    while let Some(card) = deck.draw_in(&mut rng, SPREAD) {
        assert_ne!(SPREAD & (1 << card), 0);
        drawn.push(card);
    }
    assert_eq!(drawn.len(), 4);
    // Everything else is untouched.
    assert_eq!(deck.remaining(), 12);
    assert!(deck.contains_all(TAIL));
}

meanings! {
    /// Which half of a 16-card deck an id falls in.
    enum Half {
        /// Ids 0..8.
        Low,
        /// Ids 8..16.
        High,
    }
    from_id = |id: u8| id / 8;
    cards = 16;
}

meanings! {
    /// Edge id or middle id?
    enum Zone {
        /// Ids 1..15.
        Middle,
        /// Ids 0 and 15.
        Edge,
    }
    from_id = |id: u8| if id == 0 || id == 15 { 1 } else { 0 };
    cards = 16;
}

#[test]
fn meanings_derives_masks_by_inverting_the_mapping() {
    assert_eq!(Half::High.mask(), stride_mask(8, 1, 8));
    assert_eq!(Half::Low.mask(), stride_mask(0, 1, 8));
    assert_eq!(Half::ALL, full_mask::<16>());
    assert_eq!(Zone::Edge.mask(), (1 << 15) | 1);
    assert_eq!(Zone::ALL, full_mask::<16>());
}

#[test]
fn meanings_from_id_round_trips_through_masks() {
    for id in 0..16 {
        let half = Half::from_id(id);
        assert_ne!(half.mask() & (1 << id), 0);
    }
    assert_eq!(Half::from_id(9), Half::High);
    assert_eq!(Zone::from_id(15), Zone::Edge);
    assert_eq!(Zone::from_id(7), Zone::Middle);
}

#[test]
#[should_panic(expected = "card id has no Half")]
fn meanings_from_id_panics_on_unmapped_id() {
    let _ = Half::from_id(16); // beyond the covered ids
}

#[test]
fn meanings_plug_into_deck_queries() {
    let deck = Deck::<16>::default();
    assert_eq!(deck.count_in(Half::High.mask()), 8);
    assert!(deck.contains_all(Zone::Edge.mask()));
}

#[test]
#[cfg(feature = "rand")]
fn meanings_plug_into_random_draws() {
    let mut deck = Deck::<16>::default();
    let mut rng = test_rng();
    let edge = deck.draw_in(&mut rng, Zone::Edge.mask()).unwrap();
    assert!(edge == 0 || edge == 15);
    assert_eq!(deck.count_in(Zone::Edge.mask()), 1);
}
