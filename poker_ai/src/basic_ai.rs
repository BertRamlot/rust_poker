
use poker::{round_state::RoundState, card::CardSet};
use crate::runner::ActionSupplier;

// Some super basic algorithms for playing poker
pub struct AlwaysCall {}
impl ActionSupplier for AlwaysCall {
    fn get_action(&mut self, _rs: &RoundState) -> f32 {
        return 0.0;
    }
    fn inform_finish(&mut self, _rs: &RoundState, _self_index: usize) {
    }
    fn name(&self) -> String {
        "AlwaysCall".to_owned()
    }
}

pub struct AlwaysMinRaise {}
impl ActionSupplier for AlwaysMinRaise {
    fn get_action(&mut self, _rs: &RoundState) -> f32 {
        return 1e-20;
    }
    fn inform_finish(&mut self, _rs: &RoundState, _self_index: usize) {
    }
    fn name(&self) -> String {
        "AlwaysMinRaise".to_owned()
    }
}

pub struct AlwaysFold {}
impl ActionSupplier for AlwaysFold {
    fn get_action(&mut self, _rs: &RoundState) -> f32 {
        return -1.0;
    }
    fn inform_finish(&mut self, _rs: &RoundState, _self_index: usize) {
    }
    fn name(&self) -> String {
        "AlwaysFold".to_owned()
    }
}

pub struct BasicPlayer {}
impl ActionSupplier for BasicPlayer {
    fn get_action(&mut self, rs: &RoundState) -> f32 {
        let card0 = rs.player_cards[rs.turn as usize];
        let card1 = rs.player_cards[1 + rs.turn as usize];

        let own_bet = rs.bet_chips[rs.turn as usize];
        let max_bet = rs.bet_chips.clone().into_iter().reduce(f32::max).unwrap();
        let pot_size = rs.bet_chips.iter().sum::<f32>();

        if (rs.stage == 0) || (rs.stage == 1) || (rs.stage == 2) {
            if card0.0 + card1.0 > 20 {
                return pot_size*2.0 - own_bet;
            } else if card0.0 + card1.0 > 12 {
                return 0.0;
            } else {
                if max_bet > own_bet { return -1.0 } else {return 0.0}
            }
        } else if rs.stage == 3 {
            // River, 5 cards
            let mut card_set = CardSet::new(&[
                rs.community_cards[0],
                rs.community_cards[1],
                rs.community_cards[2],
                rs.community_cards[3],
                rs.community_cards[4],
                card0,
                card1
            ]);
            card_set.canonicalize();
            let eval = card_set.evaluate();
            match eval >> 20 {
                0 => if max_bet > own_bet { return -1.0 } else {return 0.0},
                1 => return max_bet - own_bet,
                2 => return pot_size*2.0 - own_bet,
                3 => return pot_size*5.0 - own_bet,
                4 | 5 | 6 | 7 | 8 => return 1e20,
                _ => panic!("Invalid eval: {}", eval)
            }
        }
        return -1.0;
    }
    fn inform_finish(&mut self, _rs: &RoundState, _self_index: usize) {
    }
    fn name(&self) -> String {
        "BasicPlayer".to_owned()
    }
}