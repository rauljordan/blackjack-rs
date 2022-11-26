# Blackjack Rust Simulator

Runs a simulator written in Rust that can spawn N threads of blackjack games where the player
agent acts according to [basic strategy] and observes the results. It allows for customizing
the number of decks used where each deck is a standard, 52 card deck. 
Basic strategy is known to work well in casinos with good odds, however, deck size can make a difference in the house edge. 
Currently, the dealer must hit on soft 17 in the simulator, which affects RTP (return-to-player).

## Running

Install Rust, then:

```
git clone https://github.com/rauljordan/blackjack-rs && cd blackjack-rs
cargo run -- -d=6 -n=10000
```

Sample output:

```
Blackjack strategy simulator, sample game played:

Winner: Some(Dealer)
Dealer hand: [Seven, Four, Seven] = 18
Player move(s): [Stand]
Player hand(s): ([Seven, Q], []) = 17

*********************************************
* Testing effectiveness of 'basic strategy' *
*********************************************
Deck size: 6
Simulated games: 10000
Player wins: 36.96%
Dealer wins: 48.92%
Ties: 14.12%
```

## TODOs

- [ ] Customize dealer soft hit or stand on 17
- [ ] Add in bets to calculate RTP
- [ ] Allow running multiple games for an initial bet and calculate RTP
- [ ] Each game reuses a same deck, but we should give each game its own shuffled deck
