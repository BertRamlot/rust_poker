
// See: https://en.wikipedia.org/wiki/Betting_in_poker
//
// --- ORDER ---
// First round : player to the left of the blinds begins.
// Other rounds: player to the left of the dealer begins.
//
// --- BET SIZE ---
// Min bet amount: max(big blind, highest raise seen this round).
//
// --- 2-player game ---
// The normal rules for positioning the blinds do not apply when there are only two players at the table.
// The player on the button is always due the small blind, and the other player must pay the big blind.
// The player on the button is therefore the first to act before the flop, but last to act for all remaining betting rounds.
//
// --- SPLITS ---
// https://www.rookieroad.com/poker/how-do-you-split-the-pot-in-a-poker-game/

use core::panic;
use std::vec;
use std::fmt;
use std::cmp::Ordering::Equal;

use rand::prelude::*;
use crate::{card::Card, card_set::CardSet};

#[derive(Debug, PartialEq)]
pub enum RoundStage {
    PreFlop,
    Flop,
    Turn,
    River,
    Finished
}

impl RoundStage {
    fn next(&self) -> RoundStage {
        match self {
            RoundStage::PreFlop => RoundStage::Flop,
            RoundStage::Flop => RoundStage::Turn,
            RoundStage::Turn => RoundStage::River,
            RoundStage::River => RoundStage::Finished,
            RoundStage::Finished => panic!("No stage after finished!") // TODO: change this to just be idempotent?
        }
    }
}

#[derive(Debug)]
pub struct RoundState {
    pub community_cards: CardSet,
    pub button: u8,
    pub folded: u16, // Bitmask of players who have folded.
    pub stage: RoundStage,
    pub min_raise: f32,
    pub last_raise_by: u8,
    pub turn: u8,

    // Player count dependent
    pub player_count: usize,
    pub player_cards: Vec<CardSet>, // TODO: this is not ideal to use CardSet here ...
    pub bet_chips: Vec<f32>,
    pub start_chips: Vec<f32>,
    pub free_chips: Vec<f32>,
}

impl fmt::Display for RoundState {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "RoundState(\n  community_cards: '{}', stage: {:?}, min_raise: {}",
            CardSet::from(self.revealed_community_cards()),
            self.stage,
            self.min_raise,
        )?;
        for i in 0..self.player_count {
            write!(
                fmt,
                "\n    [Player {}, '{}', {:5.3}/{:5.3}] [{}|{}|{}|{}]",
                i,
                self.player_cards[i],
                self.bet_chips[i],
                self.free_chips[i],
                if self.folded & (1 << i) == 1 {"FO"} else {"  "},
                if i as u8 == self.button {"BU"} else {"  "},
                if i as u8 == self.turn {"TU"} else {"  "},
                if i as u8 == self.last_raise_by {"LR"} else {"  "},
            )?;
        }
        write!(fmt, "\n)")?;
        Ok(())
    }
}

impl Default for RoundState {
    fn default() -> RoundState {
        RoundState {
            player_count: 0,
            community_cards: "".into(),
            player_cards: vec![],
            bet_chips: vec![],
            start_chips: vec![],
            free_chips: vec![],
            
            stage: RoundStage::PreFlop,
            turn: 0,
            button: 0,
            min_raise: 1.0,
            folded: 0,
            last_raise_by: 0,
        }
    }
}

impl RoundState {
    pub fn new(free_chips: Vec<f32>) -> Self {
        let player_count = free_chips.len();
        if player_count < 2 {
            panic!("Need atleast 2 players to define a RoundState");
        }
        if player_count > 16 {
            panic!("RoundState has no support for more than 16 players");
        }

        let mut rng = thread_rng();
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rng);

        let button: usize = 0;
        let small_blind_index;
        let big_blind_index;
        let turn;
        if player_count == 2 {
            // The normal rules for positioning the blinds do not apply when there are only two players at the table.
            // The player on the button is always due the small blind, and the other player must pay the big blind.
            // The player on the button is therefore the first to act before the flop, but last to act for all remaining betting rounds.
            small_blind_index = button;
            big_blind_index = (button + 1) % player_count;
            turn = small_blind_index;
        } else {
            small_blind_index = (button + 1) % player_count;
            big_blind_index = (button + 2) % player_count;
            turn = (button + 3) % player_count;
        }

        let mut rs = RoundState {
            player_count,
            community_cards: deck[0..5].into(),
            player_cards: (0..player_count).into_iter().map(|i| deck[5+2*i..7+2*i].into()).collect(),
            bet_chips: vec![0.0; player_count],
            start_chips: free_chips.clone(),
            free_chips: free_chips,
            button: button as u8,
            turn: turn as u8,
            ..Default::default()
        };
        
        let small_blind_amount = f32::min(0.5, rs.free_chips[small_blind_index]);
        let big_blind_amount: f32 = f32::min(1.0, rs.free_chips[big_blind_index]);
        rs.bet_chips[small_blind_index] = small_blind_amount;
        rs.free_chips[small_blind_index] -= small_blind_amount;
        rs.bet_chips[big_blind_index] = big_blind_amount;
        rs.free_chips[big_blind_index] -= big_blind_amount;

        rs
    }

    pub fn revealed_community_cards(&self) -> &[Card] {
        match self.stage {
            RoundStage::PreFlop => &self.community_cards.cards[0..0],
            RoundStage::Flop => &self.community_cards.cards[0..3],
            RoundStage::Turn => &self.community_cards.cards[0..4],
            RoundStage::River | RoundStage::Finished => &self.community_cards.cards[0..5]
        }
    }

    //              Fold:                 bet_size <  0.0
    // Check/Check-raise:          0.0 <= bet_size <= check_amount
    //     Raise, All-in: check_amount <  bet_size
    pub fn do_action(&mut self, bet_size: f32) {
        if bet_size < 0.0 {
            // Fold
            self.folded |= 1 << self.turn;
        } else {
            let turn_index = self.turn as usize;

            let check_amount = f32::min(
                self.bet_chips.clone().into_iter().reduce(f32::max).unwrap_or(0.0) - self.bet_chips[turn_index],
                self.free_chips[turn_index]
            );
            let raise_amount: f32;
            if bet_size <= check_amount {
                // Check, Check-raise
                raise_amount = 0.0;
            } else {
                // Raise, All-in
                raise_amount = f32::min(
                    f32::max(bet_size - check_amount, self.min_raise),
                    self.free_chips[turn_index] - check_amount
                );
                self.min_raise = f32::max(self.min_raise, raise_amount);
            }
            let clipped_bet_size = check_amount + raise_amount;
            self.bet_chips[turn_index] += clipped_bet_size;
            self.free_chips[turn_index] -= clipped_bet_size;
            if raise_amount > 0.0 {
                self.last_raise_by = self.turn;
            }
        }
        
        // Change state, turn (& last_raise_by)
        loop {
            self.turn = (self.turn + 1) % (self.player_count as u8);
            if self.turn == self.last_raise_by {
                // Went full circle without anyone raising, go to next stage.
                self.stage = self.stage.next();
                if self.is_finished() {
                    self.finish_game();
                    break;
                } else {
                    self.turn = (self.button + 1) % (self.player_count as u8);
                    self.last_raise_by = self.turn;
                }
            }
            if (self.folded & (1 << self.turn)) == 0 && self.free_chips[self.turn as usize] > 0.0 {
                // Found next player who can act
                break;
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        return self.stage == RoundStage::Finished;
    }

    fn finish_game(&mut self) {       
        let mut winner_order: Vec<(u8, f32, i32)> = Vec::new(); // (player_idx, bet_chips, hand_strength)
        for i in 0..self.player_count {
            if self.folded & (1 << i) == 1 {
                continue;
            }
            let mut card_set = self.community_cards.clone();
            card_set.set_cards_partial(&self.player_cards[i].cards[0..2], 5);
            card_set.canonicalize();
            winner_order.push((i as u8, self.bet_chips[i], card_set.evaluate()));
        }
        
        // winner_order is sorted to have lowest bet size first for equal strength hands
        winner_order.sort_by(|(_, a_bet, _), (_, b_bet, _)| a_bet.partial_cmp(b_bet).unwrap_or(Equal));
        winner_order.sort_by(|&(_, _, a_val), &(_, _, b_val)| b_val.cmp(&a_val));

        for i in 0..winner_order.len() {
            let (fw_index, _, fw_val) = winner_order[i];
            let pot_contribution: f32 = self.bet_chips[fw_index as usize];
            if pot_contribution <= 0.0 {
                continue;
            }

            // Find between how many winners this pot is split
            let mut pot_winners = 1;
            for j in i+1..winner_order.len() {
                let (_, _, p_val) = winner_order[j];
                if fw_val != p_val {
                    break;
                }
                pot_winners += 1;
            }

            let mut pot = 0.0f32;
            let mut chips_left = false;
            // fill pot
            for j in 0..self.player_count {
                let bet_amount = f32::min(self.bet_chips[j], pot_contribution);
                self.bet_chips[j] -= bet_amount;
                if self.bet_chips[j] != 0.0 {
                    chips_left = true;
                }
                pot += bet_amount;
            }

            // distribute pot
            let winnings_per_winner = pot / (pot_winners as f32);
            for j in i..i+pot_winners {
                self.free_chips[winner_order[j].0 as usize] += winnings_per_winner;
            }

            // TODO: needed? we already select the winners pretty aggressively
            if !chips_left {
                break;
            }
        }
    }
}