use rand::Rng;
use std::thread;

// Goal: spawn tons of games of blackjack in the background using
// threads and aggregate the optimal results that won depending on dealer and player ranges.
// Aggregate the dominant strategies depending on certain game states.
//
// TODO: Make the player use actual strategy rather than random moves.
#[tokio::main]
async fn main() -> eyre::Result<()> {
    let simulation_count = 10000;
    println!("Running {} simulations", simulation_count);
    println!("========");
    println!("Sample game output...");
    let mut game = Game::new();
    game.start();
    println!("Dealer hand: {:?} = {}", game.dealer_hand, game.dealer_total);
    println!("Player hand: {:?} = {}", game.player_hand, game.player_total);
    println!("Player moves: {:?}", game.player_moves);
    println!("Winner {:?}", game.winner);
    println!("========");
    let mut handlers = vec![];
    for _ in 0..simulation_count {
        handlers.push(thread::spawn(move || {
            let mut game = Game::new();
            let winner = game.start();
            (game, winner)
        }));
    }
    let game_results: Vec<(Game, ())> = handlers.into_iter()
        .map(|handler| handler.join().unwrap())
        .collect();

    let mut player_lt_dealer_winner_total = 0;
    let mut player_lt_dealer_total = 0;
    let mut player_gt_dealer_winner_total = 0;
    let mut player_gt_dealer_total= 0;
    game_results.into_iter().for_each(|game| {
        let dealer_first = game.0.dealer_hand.first().unwrap();
        let player_first = game.0.player_hand.first().unwrap();
        if player_first < dealer_first {
            if game.0.winner == Some(Agent::Player) {
                player_lt_dealer_winner_total+= 1;
            }
            player_lt_dealer_total+= 1;
        }
        if player_first > dealer_first {
            if game.0.winner == Some(Agent::Player) {
                player_gt_dealer_winner_total+= 1;
            }
            player_gt_dealer_total+= 1;
        }
    });
    
    let player_win_lt_dealer_pct = 100.0 * (player_lt_dealer_winner_total as f64) / (player_lt_dealer_total as f64);
    let player_win_gt_dealer_pct = 100.0 * (player_gt_dealer_winner_total as f64) / (player_gt_dealer_total as f64);
    println!("Player won {:.1}% games where player first card > dealer", player_win_gt_dealer_pct);
    println!("Player won {:.1}% games where player first card < dealer", player_win_lt_dealer_pct);
    // let mut dealer_gt_player: Vec<(Arc<Game>, Option<Agent>)> = vec!();
    // let mut equal_values: Vec<(Arc<Game>, Option<Agent>)> = vec!();
    // Figure out: which move was most successful when the dealer had 
    // (1) dealer starts with a card > X, us < X
    // (2) we start with a card > X, dealer < X
    // println!("Winner {:?}", winner);
    // println!("{:?} = {}", game.dealer_hand, game.dealer_total);
    // println!("{:?}", game.player_moves);
    // println!("{:?} = {}", game.player_hand, game.player_total);
    Ok(())
}

#[derive(Debug)]
struct Game {
    winner: Option<Agent>,
    dealer_hand: Vec<u8>,
    player_moves: Vec<Move>,
    dealer_total: u8,
    player_hand: Vec<u8>,
    player_total: u8,
}

#[derive(Debug)]
enum Move {
    Double,
    Stand,
    Hit,
}

#[derive(Debug,PartialEq,Copy,Clone)]
enum Agent {
    Dealer,
    Player,
}

impl Game {
    pub fn new() -> Self {
        let dealer_hand = initial_hand();
        let player_hand = initial_hand();
        let dealer_total = (&dealer_hand).into_iter().sum();
        let player_total = (&player_hand).into_iter().sum();
        Self {
            dealer_hand,
            dealer_total,
            player_hand,
            player_total,
            player_moves: vec![],
            winner: None,
        }
    }
    pub fn start(&mut self) {
        let mut player_done = false;
        loop {
            match &self.game_ended() {
                (false, _) => {
                    // Player moves.
                    if !player_done {
                        let action = act();
                        match action {
                            Move::Hit => {
                                let card = deal();
                                self.player_hand.push(card);
                                self.player_total += card;
                            },
                            Move::Double => {
                                let card = deal();
                                self.player_hand.push(card);
                                self.player_total += card;
                                player_done = true;
                            },
                            Move::Stand => {
                                player_done = true;
                            },
                        }
                        self.player_moves.push(action);
                    } else {
                        // Dealer moves.
                        let card = deal();
                        self.dealer_hand.push(card);
                        self.dealer_total += card;
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
    pub fn game_ended(&self) -> (bool, Option<Agent>) {
        if self.player_total == 21 && self.dealer_total == 21 {
            return (true, None);
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

fn act() -> Move {
    let mut gen = rand::thread_rng();
    match gen.gen_range(0..3) {
        0 => Move::Stand,
        1 => Move::Hit,
        2 => Move::Double,
        _ => Move::Stand,
    }
}

fn deal() -> u8 {
    let mut gen = rand::thread_rng();
    gen.gen_range(1..11)
}

fn initial_hand() -> Vec<u8> {
    vec![0; 2].into_iter().map(|_| deal()).collect::<Vec<u8>>()
}