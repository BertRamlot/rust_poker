/*
See: https://en.wikipedia.org/wiki/Betting_in_poker

<--- ORDER --->
First round : player to the left of the blinds begins.
Other rounds: player to the left of the dealer begins.

<--- BET SIZE --->
Min bet amount: max(big blind, highest raise seen this round).

<--- 2-player game --->
The normal rules for positioning the blinds do not apply when there are only two players at the table. The player on the button is always due the small blind, and the other player must pay the big blind. The player on the button is therefore the first to act before the flop, but last to act for all remaining betting rounds.

<--- SPLITS --->
https://www.rookieroad.com/poker/how-do-you-split-the-pot-in-a-poker-game/

*/

use std::vec;
use std::fmt;
use std::cmp::Ordering::Equal;

use rand::prelude::*;
use crate::card::{Card, CardSet};

#[derive(Debug)]
pub struct RoundState {
    pub player_count: usize,
    pub community_cards: [Card; 5],
    pub player_cards: Vec<Card>,
    pub bet_chips: Vec<f32>,
    pub free_chips: Vec<f32>,

    pub stage: u8, // 0: pre-flop, 1: flop, 2: turn, 3: river
    pub turn: u8,
    pub button: u8,
    pub min_raise: f32,
    pub folded: u16, // Bitmask of players who have folded.
    pub last_raise_by: u8,
}

impl fmt::Display for RoundState {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let revealed_cards = match self.stage {
            0 => 0,
            1 => 3,
            2 => 4,
            3 => 5,
            4 => 5,
            _ => panic!("Invalid stage"),
        };
        fmt.write_fmt(format_args!(
            "RoundState(\n  community_cards: '{}', stage: {}, min_raise: {})",
            CardSet::from(&self.community_cards[0..revealed_cards]),
            self.stage,
            self.min_raise,
        ))?;
        for i in 0..self.player_count {
            fmt.write_fmt(format_args!(
                "\n    [Player {}, '{}', {:5.3}/{:5.3}] [{}|{}|{}|{}]",
                i,
                CardSet::from(&self.player_cards[i..i+2]),
                self.bet_chips[i],
                self.free_chips[i],
                if self.folded & (1 << i) == 1 {"FO"} else {"  "},
                if i as u8 == self.button {"BU"} else {"  "},
                if i as u8 == self.turn {"TU"} else {"  "},
                if i as u8 == self.last_raise_by {"LR"} else {"  "},
            ))?;
        }
        fmt.write_str("\n)")?;
        Ok(())
    }
}

impl Default for RoundState {
    fn default() -> RoundState {
        RoundState {
            player_count: 0,
            community_cards: [255.into(), 255.into(), 255.into(), 255.into(), 255.into()],
            player_cards: vec![],
            bet_chips: vec![],
            free_chips: vec![],
            
            stage: 0,
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
        let mut rng = thread_rng();
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rng);

        let button = 0;
        let player_count = free_chips.len();
        let small_blind_index;
        let big_blind_index;
        let turn;
        if player_count == 2 {
            /*
            The normal rules for positioning the blinds do not apply when there are only two players at the table.
            The player on the button is always due the small blind, and the other player must pay the big blind.
            The player on the button is therefore the first to act before the flop, but last to act for all remaining betting rounds.
            */
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
            community_cards: [deck[0].into(), deck[1].into(), deck[2].into(), deck[3].into(), deck[4].into()],
            player_cards: deck[5..(5+player_count*2) as usize].iter().map(|&x| x.into()).collect::<Vec<Card>>(),
            bet_chips: vec![0.0; player_count],
            free_chips: free_chips,
            button: button as u8,
            turn: turn as u8,
            ..Default::default()
        };
        
        let small_blind_amount = f32::min(0.5, rs.free_chips[small_blind_index]);
        let big_blind_amount = f32::min(1.0, rs.free_chips[big_blind_index]);
        rs.bet_chips[small_blind_index] = small_blind_amount;
        rs.free_chips[small_blind_index] -= small_blind_amount;
        rs.bet_chips[big_blind_index] = big_blind_amount;
        rs.free_chips[big_blind_index] -= big_blind_amount;

        rs
    }

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
                self.stage += 1;
                self.turn = (self.button + 1) % (self.player_count as u8);
                self.last_raise_by = self.turn;
                break;
            }
            if (self.folded & (1 << self.turn)) == 0 && self.free_chips[self.turn as usize] > 0.0 {
                // Found next player who can act
                break;
            }
        }
    }

    pub fn is_finished(&self) -> bool {
        return self.stage == 4;
        /*
        if self.stage >= 4 {
            return true;
        }
        let mut actable_players = 0;
        for i in 0..self.player_count {
            if (self.folded & (1 << i) == 0) && (self.free_chips[i] > 0.0) {
                actable_players += 1;
                if actable_players == 2 {
                    return false;
                }
            }
        }
        return true;
         */
    }

    pub fn finish_game(&mut self) {
        let mut cards: [Card; 7] = [self.community_cards[0], self.community_cards[1], self.community_cards[2], self.community_cards[3], self.community_cards[4], 255.into(), 255.into()];
        let mut winner_order: Vec<(u8, f32, i32)> = Vec::new();
        for i in 0..self.player_count {
            if self.folded & (1 << i) == 1 {
                continue;
            }
            cards[5] = self.player_cards[i*2];
            cards[6] = self.player_cards[i*2+1];
            let mut card_set = CardSet::new(&cards);
            card_set.canonicalize();
            winner_order.push((i as u8, self.bet_chips[i], card_set.evaluate()));
        }
        
        // TODO: special case for only 1 winner? (for speed)
        // winner_order is sorted to have lowest bet size first for equal strength hands
        winner_order.sort_by(|(_, a_bet, _), (_, b_bet, _)| a_bet.partial_cmp(b_bet).unwrap_or(Equal));
        winner_order.sort_by(|&(_, _, a_val), &(_, _, b_val)| b_val.cmp(&a_val));

        for i in 0..winner_order.len() {
            let (fw_index, _, fw_val) = winner_order[i];
            // Find how many winners for this pot
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
            let pot_contribution_per = self.bet_chips[fw_index as usize];
            // fill pot
            for n in 0..self.player_count {
                let bet_amount = f32::min(self.bet_chips[n], pot_contribution_per);
                self.bet_chips[n] -= bet_amount;
                if self.bet_chips[n] != 0.0 {
                    chips_left = true;
                }
                pot += bet_amount;
            }
            // println!("Pot: {}, {}..={}, bets: {:?}", pot, i, i+pot_winners-1, self.bet_chips);

            // distribute pot
            let pot_winnings_per_player = pot / (pot_winners as f32);
            for j in i..i+pot_winners {
                self.free_chips[winner_order[j].0 as usize] += pot_winnings_per_player;
            }

            // TODO: needed? we already select the winners pretty aggressively
            if !chips_left {
                break;
            }
        }
    }
}