use eframe::{egui, epi};
use rand::{seq::SliceRandom, thread_rng};

#[derive(Clone, Copy)]
enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Clone, Copy)]
enum Value {
    Number(u8), // 2-10
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Clone)]
struct Card {
    value: Value,
    suit: Suit,
}

impl Card {
    fn display(&self) -> String {
        let value_str = match &self.value {
            Value::Number(num) => num.to_string(),
            Value::Jack => "J".to_string(),
            Value::Queen => "Q".to_string(),
            Value::King => "K".to_string(),
            Value::Ace => "A".to_string(),
        };
        format!("{} of {:?}", value_str, self.suit)
    }

    fn value(&self) -> u8 {
        match self.value {
            Value::Number(num) => num,
            Value::Jack | Value::Queen | Value::King => 10,
            Value::Ace => 11, // Ace handling for 1 or 11 will be in hand value calculation
        }
    }
}

struct BlackjackApp {
    deck: Vec<Card>,
    player_hands: Vec<Vec<Card>>,
    dealer_hand: Vec<Card>,
    current_hand: usize,
    game_state: GameState,
    player_bets: Vec<usize>,
    total_money: usize,
}

enum GameState {
    Betting,
    PlayerTurn,
    DealerTurn,
    GameOver(String),
}

impl Default for BlackjackApp {
    fn default() -> Self {
        Self::new()
    }
}

impl BlackjackApp {
    fn new() -> Self {
        let mut app = BlackjackApp {
            deck: Vec::new(),
            player_hands: vec![Vec::new()],
            dealer_hand: Vec::new(),
            current_hand: 0,
            game_state: GameState::Betting,
            player_bets: vec![10], // Initial bet
            total_money: 100, // Starting money
        };
        app.new_round();
        app
    }

    fn create_deck() -> Vec<Card> {
        let suits = [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades];
        let values = [
            Value::Number(2), Value::Number(3), Value::Number(4), Value::Number(5), Value::Number(6),
            Value::Number(7), Value::Number(8), Value::Number(9), Value::Number(10),
            Value::Jack, Value::Queen, Value::King, Value::Ace,
        ];
        let mut deck = Vec::new();

        for &suit in suits.iter() {
            for &value in values.iter() {
                deck.push(Card { value, suit });
            }
        }

        deck
    }

    fn can_split(hand: &[Card]) -> bool {
        if hand.len() != 2 {
            return false;
        }
        let first_card_value = hand[0].value();
        let second_card_value = hand[1].value();

        first_card_value == second_card_value
    }

    fn new_round(&mut self) {
        self.deck = Self::create_deck();
        self.deck.shuffle(&mut thread_rng());
        self.player_hands = vec![vec![self.deck.pop().unwrap(), self.deck.pop().unwrap()]];
        self.dealer_hand = vec![self.deck.pop().unwrap(), self.deck.pop().unwrap()];
        self.current_hand = 0;
        self.game_state = GameState::PlayerTurn;
    }

    fn hit(&mut self) {
        if let Some(card) = self.deck.pop() {
            self.player_hands[self.current_hand].push(card);
            if self.calculate_hand_value(&self.player_hands[self.current_hand]) > 21 {
                // Player busts
                self.stand(); // Move to next hand or dealer's turn
            }
        } else {
            eprintln!("Deck depleted. Unable to draw more cards.");
        }
    }

    fn stand(&mut self) {
        if self.current_hand + 1 < self.player_hands.len() {
            self.current_hand += 1; // Move to the next hand if any
        } else {
            self.game_state = GameState::DealerTurn; // Move to dealer's turn
            self.dealer_turn();
        }
    }

    fn double_down(&mut self) {
        if self.total_money >= self.player_bets[self.current_hand] {
            self.total_money -= self.player_bets[self.current_hand];
            self.player_bets[self.current_hand] *= 2;
            self.hit();
            if self.calculate_hand_value(&self.player_hands[self.current_hand]) <= 21 {
                self.stand();
            }
        } else {
            eprintln!("Insufficient funds to double down.");
        }
    }

    fn split(&mut self) {
        if !Self::can_split(&self.player_hands[self.current_hand]) || self.total_money < self.player_bets[self.current_hand] {
            eprintln!("Cannot split.");
            return;
        }

        let hand_to_split = self.player_hands[self.current_hand].clone();
        let bet_for_new_hand = self.player_bets[self.current_hand];

        self.total_money -= bet_for_new_hand;
        self.player_bets.push(bet_for_new_hand);

        // Remove one card from the current hand and start a new hand with it
        let card_for_new_hand = hand_to_split[1].clone();
        self.player_hands[self.current_hand].pop();
        self.player_hands[self.current_hand].push(self.deck.pop().unwrap());
        self.player_hands.push(vec![card_for_new_hand, self.deck.pop().unwrap()]);
    }

    fn dealer_turn(&mut self) {
        while self.calculate_hand_value(&self.dealer_hand) < 17 {
            if let Some(card) = self.deck.pop() {
                self.dealer_hand.push(card);
            } else {
                break; // Dealer stops if deck is depleted
            }
        }
        self.evaluate_game_outcomes();
    }

    fn calculate_hand_value(hand: &[Card]) -> usize {
        let mut value = 0;
        let mut aces = 0;

        for card in hand {
            match card.value {
                Value::Ace => aces += 1,
                _ => value += card.value() as usize,
            }
        }

        // Add Ace value(s) considering the best outcome
        for _ in 0..aces {
            if value + 11 > 21 {
                value += 1; // Use Ace as 1
            } else {
                value += 11; // Use Ace as 11, potentially
            }
        }

        value
    }

    fn evaluate_game_outcomes(&mut self) {
        let dealer_value = Self::calculate_hand_value(&self.dealer_hand);
        let dealer_bust = dealer_value > 21;
        let mut message = String::from("Round Over: ");

        for (index, hand) in self.player_hands.iter().enumerate() {
            if Self::calculate_hand_value(hand) > 21 {
                message.push_str(&format!("Hand {} Busted. ", index + 1));
                continue;
            }

            let hand_value = Self::calculate_hand_value(hand);
            if hand_value > 21 || (!dealer_bust && dealer_value > hand_value) {
                message.push_str(&format!("Hand {} Lost. ", index + 1));
            } else if dealer_bust || hand_value > dealer_value {
                message.push_str(&format!("Hand {} Won! ", index + 1));
                self.total_money += self.player_bets[index] * 2; // Win double the bet
            } else if hand_value == dealer_value {
                message.push_str(&format!("Hand {} Push. ", index + 1));
                self.total_money += self.player_bets[index]; // Return the bet
            }
        }

        self.game_state = GameState::GameOver(message);
    }
}

impl epi::App for BlackjackApp {
    fn name(&self) -> &str {
        "Blackjack"
    }

    fn update(&mut self, ctx: &egui::CtxRef, _frame: &mut epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Blackjack");
            match self.game_state {
                GameState::Betting => {
                    if ui.button("Place Bet and Start").clicked() {
                        self.new_round();
                    }
                },
                GameState::PlayerTurn => {
                    ui.label(format!("Current Hand: {:?}", self.player_hands[self.current_hand].iter().map(|c| c.display()).collect::<Vec<_>>()));
                    if ui.button("Hit").clicked() {
                        self.hit();
                    }
                    if ui.button("Stand").clicked() {
                        self.stand();
                    }
                    if ui.button("Double Down").clicked() {
                        self.double_down();
                    }
                    if self.player_hands[self.current_hand].len() == 2 && Self::can_split(&self.player_hands[self.current_hand]) {
                        if ui.button("Split").clicked() {
                            self.split();
                        }
                    }
                },
                GameState::DealerTurn => {
                    ui.label("Dealer's turn...");
                },
                GameState::GameOver(ref message) => {
                    ui.label(message);
                    if ui.button("Play Again").clicked() {
                        self.new_round();
                    }
                },
            }
        });
    }
}
