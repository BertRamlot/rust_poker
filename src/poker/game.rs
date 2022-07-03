

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

use rand::prelude::*;
use std::cmp::Ordering::Equal;
use super::card::Card;
use crate::poker::cardset::{CardSet};

pub struct RoundState {
    pub(crate) player_count: usize,
    pub(crate) stage: u8, // 0: pre-flop, 1: flop, 2: turn, 3: river
    pub(crate) turn: u8,
    pub(crate) min_raise: f32,
    pub(crate) community_cards: [Card; 5],

    pub(crate) player_cards: Vec<Card>,
    pub(crate) bet_chips: Vec<f32>,
    pub(crate) free_chips: Vec<f32>,
    pub(crate) folded: u16, // Bitmask of players who have folded.
}

impl RoundState {
    pub fn new(player_count: usize) -> Self {
        let mut rng = thread_rng();
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rng);

        let rs = RoundState {
            player_count,
            stage: 0,
            turn: 0,
            min_raise: 1.0,
            community_cards: [deck[0].into(), deck[1].into(), deck[2].into(), deck[3].into(), deck[4].into()],
            player_cards: deck[5..(5+player_count*2) as usize].iter().map(|&x| x.into()).collect::<Vec<Card>>(),
            bet_chips: vec![0.0; player_count],
            free_chips: vec![0.0; player_count],
            folded: 0,
        };
        rs
    }

    pub fn is_finished(&self) -> bool {
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