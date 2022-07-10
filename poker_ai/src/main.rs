use std::cell::RefCell;
use std::rc::Rc;
use std::vec;

use poker_ai::runner::{ActionSupplier, benchmark_players};
use poker_ai::cfrm::{CFRMSupplier, CFRMBackbone};
use poker_ai::basic_ai::{AlwaysCall, AlwaysMinRaise, AlwaysFold, BasicPlayer};

fn main() {
    println!("Starting test");
    train_and_test();
    // standard_test();
}

#[allow(dead_code)]
fn train_and_test<'a>() {
    let cfrm_backbone = Rc::new(RefCell::new(Box::new(CFRMBackbone::new())));
    let mut all_players: Vec<Box<dyn ActionSupplier + 'a>> = vec![
        Box::new(AlwaysCall{}),
        Box::new(AlwaysMinRaise{}),
        Box::new(AlwaysFold{}),
        Box::new(BasicPlayer{}),
    ];
    let non_cfrm_player_count = all_players.len();
    all_players.push(Box::new(CFRMSupplier::new(cfrm_backbone.clone(), "LIVE".to_owned())));

    for itt in 0..10 {
        /*
        {
            let backbone = cfrm_backbone.borrow();

            let info_community_id = CardSet::from("").as_canonical().identifier();
            let info_private_id = CardSet::from([Card::from("As"), Card::from("Ah")].as_slice()).as_canonical().identifier();
            let info_pub_bucket = backbone.public_buckets.get(&info_community_id);
            if info_pub_bucket.is_some() {
                let info_priv_bucket = info_pub_bucket.unwrap().private_buckets.get(&info_private_id);
                if info_priv_bucket.is_some() {
                    println!("{:?}", info_priv_bucket);
                }
            }
            println!("Nodes: {}", backbone.node_count);
        }
        */

        let train_players: Vec<usize> = vec![all_players.len()-2, all_players.len()-1];
        // Train the CFRM player and add to prev_cfrm_players
        let train_run_result = benchmark_players(
            all_players.as_mut(),
            train_players.as_slice(),
            2,
            10000000
        );

        // let itt_backbone = Rc::new(RefCell::new(Box::new((**cfrm_backbone.borrow_mut()).clone())));
        // all_players.push(Box::new(CFRMSupplier::new(itt_backbone.clone(), format!("{}", itt).to_owned())));

        // Run a test to see how good new itt is
        let test_players_indices: Vec<usize> = (0..all_players.len()).collect();
        let test_run_result = benchmark_players(
            all_players.as_mut(), // all_players.as_mut(),
            test_players_indices.as_slice(), // train_players.as_slice(),
            2, 
            500000
        );
        print!("Finished training itt {} (took {:.2}s)", itt, train_run_result.elapsed.as_secs_f32());
        println!(" test_run_result:\n{}",  test_run_result);
    }
    println!("Finished training/testing");
}

#[allow(dead_code)]
fn standard_test() {
    let mut all_players: Vec<Box<dyn ActionSupplier>> = vec![
        Box::new(AlwaysCall{}),
        Box::new(AlwaysMinRaise{}),
        Box::new(AlwaysFold{}),
        Box::new(BasicPlayer{}),
    ];
    let mut player_indices: Vec<usize> = vec![];
    (0..all_players.len()).into_iter().for_each(|i| player_indices.push(i));

    for ts in 2..=4 {
        let res = benchmark_players(
            all_players.as_mut(),
            player_indices.as_slice(),
            ts,
            1 * 1000000
        );
        println!("Finished test, results:\n{}", res);
    }
}
