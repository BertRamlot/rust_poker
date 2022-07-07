
#[cfg(test)]
mod game_test {
    use poker::round_state::RoundState;

    #[test]
    pub fn test_game_finish() {
        let mut rs1 = RoundState {
            player_count: 8,
            stage: 3,
            community_cards: ["As".into(), "Ks".into(), "5h".into(), "6c".into(), "8c".into()],
            player_cards: vec![
                "Ah".into(), "4h".into(), // 0 2nd
                "Ac".into(), "3c".into(), // 1 2nd
                "Kh".into(), "5s".into(), // 2 1st
                "Kc".into(), "5c".into(), // 3 1st
                "Kd".into(), "5d".into(), // 4 1st
                "2s".into(), "3s".into(), // 5 5th
                "8h".into(), "4d".into(), // 6 4th
                "6h".into(), "2d".into(), // 7 3th
            ],
            bet_chips:  vec![3.0, 50.0,  0.0, 10.0, 100.0, 110.0, 15.0, 130.0],
            free_chips: vec![0.0,  0.0, 25.0,   0.0,  0.0,  0.0,  10.0,  20.0],
            folded: (1 << 2) | (1 << 6),
            ..Default::default()
        };
        while !rs1.is_finished() {
            rs1.do_action(0.0);
        }
        assert!(rs1.is_finished(), "rs1 should be finished");
        rs1.finish_game();

        assert_eq!(rs1.free_chips, vec![0.0, 0.0, 25.0, 31.5, 346.5, 0.0, 10.0, 60.0], "Wrong distribution of chips");
    }
}