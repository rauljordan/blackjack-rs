#[macro_use]
extern crate lazy_static;

use std::sync::{Arc,Mutex};
use std::thread;
use rand::seq::SliceRandom;
use rand::thread_rng;

use structopt::StructOpt;

mod strategy;

use strategy::BASIC_STRATEGY;

#[derive(Debug, StructOpt)]
pub struct Opt {
    // 6 decks for the game (used by Vegas tables).
    #[structopt(short = "d", default_value = "6")]
    num_decks: usize,
    // Number of games to simulate.
    #[structopt(short = "n", default_value = "10000")]
    simulation_count: usize,
}

// Goal: spawn tons of games of blackjack in the background using
// threads and aggregate the results that won depending on dealer and player ranges.
// Observe the performance of the commonly touted "basic strategy" from the results.
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let opts = Opt::from_args();
    let deck = Deck::new(opts.num_decks);
    let cards = Arc::new(Mutex::new(deck.cards.into_iter().cycle()));

    let mut handlers = vec![];
    for _ in 0..opts.simulation_count {
        let mut cards = cards.clone();
        handlers.push(thread::spawn(move || {
            let mut game = Game::new(&mut cards);
            game.start();
            GameResult::from(game)
        }));
    }
    let game_results: Vec<GameResult> = handlers.into_iter()
        .map(|handler| handler.join().unwrap())
        .collect();

    let mut player_w: f64 = 0.0;
    let mut dealer_w: f64 = 0.0;
    let mut draw: f64 = 0.0;
    let tot = game_results.len() as f64;
    game_results
        .into_iter()
        .for_each(|g| {
            match g.winner {
                Some(Agent::Player) => player_w += 1.0,
                Some(Agent::Dealer) => dealer_w += 1.0,
                None => draw += 1.0,
            }
        });

    println!("********************************************");
    println!("*                                          *'");
    println!("* Testing effectiveness of 'basic strategy' *'");
    println!("*                                          *'");
    println!("********************************************");
    println!("Deck size: {}", opts.num_decks);
    println!("Simulated games: {}", opts.simulation_count);
    println!("Player wins: {}%", player_w / tot * 100.0);
    println!("Dealer wins: {}%", dealer_w / tot * 100.0);
    println!("Ties: {}%", draw / tot * 100.0);
    Ok(())
}

#[derive(Debug,Clone,Copy)]
pub enum Card {
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

// Turns a card into its u8 representation, as several face cards
// can all map to 10. For now, treats aces as mapping to 11.
impl From<&Card> for u8 {
    fn from(c: &Card) -> Self {
        match c {
            Card::Two => 2,
            Card::Three => 3,
            Card::Four => 4,
            Card::Five => 5,
            Card::Six => 6,
            Card::Seven => 7,
            Card::Eight => 8,
            Card::Nine => 9,
            Card::Ten => 10,
            Card::J => 10,
            Card::Q => 10,
            Card::K => 10,
            Card::A => 11,
        }
    }
}

// Creates a deck instance out of an allowed card set and shuffles it.
#[derive(Debug)]
pub struct Deck {
    cards: Vec<Card>,
}

impl Deck {
    pub fn new(num_decks: usize) -> Self {
        let card_set = [
            Card::A,
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
        ];
        // Create multiple multiple decks if desired.
        let mut cards: Vec<Card> = card_set
            .iter()
            .cycle()
            .take(card_set.len()*num_decks)
            .cloned()
            .collect();

        cards.shuffle(&mut thread_rng());
        
        Self {
            cards,
        } 
    }
    pub fn shuffle(&mut self) {
        self.cards.shuffle(&mut thread_rng());
    }
}

#[derive(Debug,Clone)]
pub enum Move {
    Double,
    Stand,
    Hit,
}

#[derive(Debug,PartialEq,Copy,Clone)]
pub enum Agent {
    Dealer,
    Player,
}

#[derive(Debug)]
pub struct Game<'a, T: Iterator> {
    deck: &'a mut Arc<Mutex<T>>,
    dealer_hand: Vec<Card>,
    dealer_total: u8,
    player_moves: Vec<Move>,
    player_hand: Vec<Card>,
    player_total: u8,
    dealer_beats_player: bool,
    winner: Option<Agent>,
}

impl <'a, T> Game<'a, T> where T: Iterator<Item=Card> {
    pub fn new(
        cards: &'a mut Arc<Mutex<T>>,
    ) -> Self {
        let dealer_hand = take_two(cards);
        let player_hand = take_two(cards);
        let dealer_total = u8::from(dealer_hand.first().unwrap());
        let player_total = hand_sum(&player_hand);
        Self {
            deck: cards,
            dealer_hand,
            dealer_total,
            player_hand,
            player_total,
            player_moves: vec![],
            dealer_beats_player: false,
            winner: None,
        }
    }
    pub fn start(&mut self) {
        let mut player_done = false;
        let mut dealer_revealed = false;
        loop {
            match &self.game_ended() {
                (false, _) => {
                    // Player moves.
                    if !player_done {
                        let action = self.act();
                        match action {
                            Move::Hit => {
                                let card = self.next_card();
                                self.player_hand.push(card);
                                self.player_total += u8::from(&card);
                            },
                            Move::Double => {
                                let card = self.next_card();
                                self.player_hand.push(card);
                                self.player_total += u8::from(&card);
                                player_done = true;
                            },
                            Move::Stand => {
                                player_done = true;
                            },
                        }
                        self.player_moves.push(action);
                    } else {
                        if !dealer_revealed {
                            self.dealer_total += u8::from(self.dealer_hand.last().unwrap());
                            dealer_revealed = true;
                            continue;
                        }
                        // Dealer moves.
                        let card = self.next_card();
                        self.dealer_hand.push(card);
                        self.dealer_total += u8::from(&card);
                        if self.dealer_total <= 21 && self.dealer_total > self.player_total {
                            self.dealer_beats_player = true;
                        }
                    }
                },
                (true, None) => {
                    self.winner = None;
                    return;
                }
                (true, Some(agent)) => {
                    self.winner = Some(agent.clone());
                    return;
                }
            }
        }
    }
    pub fn next_card(&self) -> Card {
        let mut deck = self.deck
            .lock()
            .unwrap();
        deck.next().unwrap()
    }
    pub fn act(&self) -> Move {
        let dealer_up_card = u8::from(self.dealer_hand.first().unwrap());
        let player_sum = hand_sum(&self.player_hand);

        if player_sum < 5 {
            return Move::Hit;
        }

        // Always stand if sum > 17.
        if player_sum > 17 {
            return Move::Stand;
        }

        let key = format!("{},{}", player_sum, dealer_up_card);
        let strat = BASIC_STRATEGY.lock().unwrap();
        match strat.get(&key.as_str()) {
            Some(action) => action.clone(),
            None => panic!("no move found for situation {}", key)
        }
    }
    pub fn game_ended(&self) -> (bool, Option<Agent>) {
        if self.player_total == self.dealer_total {
            return (true, None);
        }
        if self.dealer_beats_player {
            return (true, Some(Agent::Dealer)); 
        }
        if self.player_total == 21 {
            return (true, Some(Agent::Player));
        }
        if self.dealer_total == 21 {
            return (true, Some(Agent::Dealer));
        }
        if self.player_total > 21 {
            return (true, Some(Agent::Dealer)); 
        }
        if self.dealer_total > 21 {
            return (true, Some(Agent::Player)); 
        }
        return (false, None);
    }
}

// Simple summary of the game for displaying to the user.
pub struct GameResult {
    dealer_hand: Vec<Card>,
    player_hand: Vec<Card>,
    player_moves: Vec<Move>,
    winner: Option<Agent>,
}

impl <'a, T> From<Game<'a, T>> for GameResult
    where T: Iterator<Item=Card> {
    fn from(g: Game<'a, T>) -> Self {
        Self {
            dealer_hand: g.dealer_hand,
            player_hand: g.player_hand,
            player_moves: g.player_moves,
            winner: g.winner,
        } 
    }
}

// Get the sum of cards in hand.
pub fn hand_sum(hand: &Vec<Card>) -> u8 {
    hand.into_iter().map(|c| u8::from(c)).sum()
}

// Take two cards from the deck iterator.
pub fn take_two<T: Iterator<Item=Card>>(cards: &mut Arc<Mutex<T>>) -> Vec<Card> {
    let binding = cards.clone();
    let mut deck = binding.lock().unwrap();
    vec![deck.next().unwrap(), deck.next().unwrap()]
}
