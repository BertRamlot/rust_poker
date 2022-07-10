use rand::{thread_rng, Rng};
use std::{collections::HashMap, cell::RefCell, rc::Rc, borrow::BorrowMut};

use poker::{round_state::RoundState, card::CardSet};
use crate::runner::ActionSupplier;


#[derive(Clone, Debug)]
pub struct Action {
    bet_size: f32,
    regret: f32,
    chance: f32,
    occs: u64,
}
impl Default for Action {
    fn default() -> Self {
        Action {
            bet_size: 0.0,
            regret: 0.0,
            chance: 0.0,
            occs: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrivateBucket {
    actions: Vec<Rc<RefCell<Action>>>,
    occs_since_last_update: u64,
}
impl PrivateBucket {
    fn new() -> PrivateBucket {
        let mut pb = PrivateBucket {
            occs_since_last_update: 0,
            actions: vec![
                Rc::new(RefCell::new(Action {
                    bet_size: -1.0,
                    ..Default::default()
                })),
                Rc::new(RefCell::new(Action {
                    bet_size: 0.0,
                    ..Default::default()
                })),
                Rc::new(RefCell::new(Action {
                    bet_size: 0.0001,
                    ..Default::default()
                })),
                Rc::new(RefCell::new(Action {
                    bet_size: 10.0,
                    ..Default::default()
                })),
                Rc::new(RefCell::new(Action {
                    bet_size: 1000.0,
                    ..Default::default()
                })),
            ],
        };
        let chance = 1.0 / pb.actions.len() as f32;
        pb.actions.iter_mut().for_each(|a| {
            let mut a = (**a.borrow_mut()).borrow_mut();
            a.chance = chance;
        });
        pb
    }

    fn sample_action(&mut self) -> Rc<RefCell<Action>> {
        let mut rng = thread_rng();
        let random: f32 = rng.gen_range(0.0..1.0);

        let mut cumulative_sum = 0.0;
        for i in 0..self.actions.len()-1 {
            let a = self.actions[i].borrow();
            cumulative_sum += a.chance;
            if cumulative_sum >= random {
                return self.actions[i].clone();
            }
        }
        return self.actions[self.actions.len()-1].clone();
    }

    fn re_calculate_strategy(&mut self) {
        let mut sum: f32 = 0.0;
        let mut pos_regret: Vec<f32> = vec![0.0; self.actions.len()];
        for i in 0..self.actions.len() {
            let clamped_regret;
            if self.actions[i].borrow().occs < 1000 {
                // Try to give each action a fair shot by taking more than 1 sample
                // and not shunning the action completely for 1 bad sample.
                clamped_regret = 1.0;
            } else{
                clamped_regret = f32::max(0.0, self.actions[i].borrow().regret / self.actions[i].borrow().occs as f32);            
            }
            pos_regret[i] = clamped_regret;
            sum += clamped_regret;
        }

        if sum > 0.0 {
            for i in 0..self.actions.len() {
                let mut a = (**self.actions[i].borrow_mut()).borrow_mut();
                a.chance = pos_regret[i] / sum;
            }
            // Sort so that the highest chance is first. This (should) speed up sampling.
            self.actions.sort_by(|a, b| b.borrow().chance.partial_cmp(&a.borrow().chance).unwrap());
        } else {
            // No positive regret, make actions frequency uniformly distributed
            let uniform_chance: f32 = 1.0 / (self.actions.len() as f32);
            self.actions.iter_mut().for_each(|action| {
                let mut a = (**action.borrow_mut()).borrow_mut();
                a.chance = uniform_chance;
            });
        }
    }
}

#[derive(Clone, Debug)]
pub struct PublicBucket {
    pub private_buckets: HashMap<u64, PrivateBucket>,
}
impl PublicBucket {
    fn new() -> PublicBucket {
        PublicBucket {
            private_buckets: HashMap::new(),
        }
    }

    fn get_private_bucket(&mut self, rs: &RoundState) -> &mut PrivateBucket {
        let hand_cards = [
            rs.player_cards[(2*rs.turn+0) as usize],
            rs.player_cards[(2*rs.turn+1) as usize]
        ];
        let key = CardSet::new(&hand_cards).identifier();
        self.private_buckets.entry(key).or_insert_with(|| PrivateBucket::new())
    }
}

pub struct CFRMSupplier {
    backbone: Rc<RefCell<Box<CFRMBackbone>>>,
    // Current info
    path: Vec<Rc<RefCell<Action>>>,
    // Tag
    tag: String,
}

#[derive(Clone, Debug)]
pub struct CFRMBackbone {
    pub public_buckets: HashMap<u64, Box<PublicBucket>>,
    pub node_count: u64,
}
impl CFRMBackbone {
    pub fn new() -> CFRMBackbone {
        CFRMBackbone {
            public_buckets: HashMap::new(),
            node_count: 0,
        }
    }
    
    fn get_public_bucket(&mut self, rs: &RoundState) -> &mut Box<PublicBucket> {
        let key = CardSet::from(rs.get_revealed_community_cards()).as_canonical().identifier();
        self.public_buckets.entry(key).or_insert_with(|| {
            self.node_count += 1;
            Box::new(PublicBucket::new())
        })
    }
}

impl ActionSupplier for CFRMSupplier {
    fn get_action(&mut self, rs: &RoundState) -> f32 {
        let mut backbone = (**self.backbone.borrow_mut()).borrow_mut();
        let public_bucket = backbone.get_public_bucket(rs);
        let mut private_bucket = public_bucket.get_private_bucket(rs);
        if private_bucket.occs_since_last_update >= 10000 {
            private_bucket.re_calculate_strategy();
            private_bucket.occs_since_last_update = 0;
        }
        private_bucket.occs_since_last_update += 1;
        let mut sampled_action = private_bucket.sample_action();
        self.path.push(sampled_action.clone());
        let sampled_action_borrowed = (**sampled_action.borrow_mut()).borrow_mut();

        return sampled_action_borrowed.bet_size;
    }

    fn inform_finish(&mut self, rs: &RoundState, self_index: usize) {
        self.path.iter_mut().for_each(|action| {
            let mut action_mut = (**action.borrow_mut()).borrow_mut();
            action_mut.regret -= rs.free_chips[self_index]-rs.start_chips[self_index];
            action_mut.occs += 1;
            // println!("action: {:?}", action_mut);
        });
        self.path = Vec::new();
    }

    fn name(&self) -> String {
        if self.tag.is_empty() {
            return "CFRM".to_owned();
        } else {
            return format!("CFRM[{}]", self.tag);
        }
    }
}

impl CFRMSupplier {
    pub fn new(cfrm_backbone: Rc<RefCell<Box<CFRMBackbone>>>, tag: String) -> CFRMSupplier {
        CFRMSupplier {
            backbone: cfrm_backbone,

            path: Vec::new(),

            tag,
        }
    }
}
