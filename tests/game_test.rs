
#[cfg(test)]
mod game_test {
    use poker::round_state::{RoundState, RoundStage};

    #[test]
    pub fn test_chip_distribution() {
        let mut round_state = RoundState {
            player_count: 8,
            stage: RoundStage::PreFlop,
            community_cards: "As Ks 5h 6c 8c".into(),
            player_cards: vec![
                "Ah 4h", // 0 2nd
                "Ac 3c", // 1 2nd
                "Kh 5s", // 2 1st
                "Kc 5c", // 3 1st
                "Kd 5d", // 4 1st
                "2s 3s", // 5 5th
                "8h 4d", // 6 4th
                "6h 2d", // 7 3th
            ].iter().map(|&s| s.into()).collect(),
            bet_chips:  vec![3.0, 50.0,  6.0, 10.0, 15.0, 110.0, 15.0, 130.0],
            free_chips: vec![0.0,  0.0, 25.0,  0.0,  0.0,  0.0,  10.0,  20.0],
            folded: (1 << 2) | (1 << 6),
            ..Default::default()
        };
        while !round_state.is_finished() {
            round_state.do_action(0.0);
        }
        assert!(round_state.is_finished(), "rs1 should be finished");

        assert_eq!(
            round_state.free_chips,
            vec![0.0, 105.0, 40.0, 27.0, 52.0, 0.0, 10.0, 160.0],
            "Wrong distribution of chips"
        );
    }
}