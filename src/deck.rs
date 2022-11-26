use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::Move;

#[derive(Debug,Clone)]
pub enum Card {
    One,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    J,
    Q,
    K,
    A,
}

pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new(num_decks: usize) -> Self {
        let card_set = [
            Card::One,
            Card::Two,
            Card::Three,
            Card::Four,
            Card::Five,
            Card::Six,
            Card::Seven,
            Card::Eight,
            Card::Nine,
            Card::Ten,
            Card::J,
            Card::Q,
            Card::K,
            Card::A,
        ];
        // Create multiple multiple decks if desired.
        let cards: Vec<Card> = card_set
            .iter()
            .cycle()
            .take(card_set.len()*num_decks)
            .cloned()
            .collect();
        Self {
            cards,
        } 
    }
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }
}



























