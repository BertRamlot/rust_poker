extern crate poker;
use poker::round_state::RoundState;

use colored::{Colorize, ColoredString};
use rand::{prelude::SliceRandom, thread_rng};
use std::time::{Duration, Instant};


trait ActionSupplier {
    fn get_action(&self, rs: &RoundState) -> f32;
    fn name(&self) -> &str;
}


struct FullTestResults<'a> {
    players: &'a [Box<dyn ActionSupplier>],
    games_played: Vec<[u64; 16]>,
    avg_win: Vec<[f32; 16]>, // avg_win[i][j] -> avg big_blind win for [PLAYER i] playing in [POSITION j]
    table_size: usize,
    elapsed: Duration,
}
impl std::fmt::Display for FullTestResults<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name_length = self.players.iter().map(|p| p.name().len()).max().unwrap();

        // Calculate total games played
        let total_games_played: u64 = self.games_played.iter().map(|a| a.iter().sum::<u64>()).sum::<u64>()/(self.table_size as u64);
        write!(f, "Simulated {} games in {:.2} ({:.2} games per second)\n",
            total_games_played,
            self.elapsed.as_secs_f32(),
            (total_games_played as f32)/self.elapsed.as_secs_f32()
        )?;

        // Calculate aggregate preformance for each player
        let mut aggregate_preformance = vec![0.0; self.players.len()];
        for i in 0..self.players.len() {
            for j in 0..self.table_size {
                aggregate_preformance[i] += self.avg_win[i][j];
            }
        }

        // Find best and worst preformers per position
        let mut best_preformer: Vec<usize> = Vec::new();
        let mut worst_preformer: Vec<usize> = Vec::new();
        for j in 0..self.table_size {
            let mut best_index: usize = 0;
            let mut worst_index: usize = 0;
            for i in 0..self.players.len() {
                if self.avg_win[i][j] > self.avg_win[best_index][j] {
                    best_index = i;
                }
                if self.avg_win[i][j] < self.avg_win[worst_index][j] {
                    worst_index = i;
                }
            }
            best_preformer.push(best_index);
            worst_preformer.push(worst_index);
        }

        // Write header
        write!(f, "  {:^width$}", "Player", width=name_length)?;
        write!(f, "{:^width$}", "TOTAL", width=7)?;
        for i in 0..self.table_size {
            write!(f, " POS={:width$}", i, width=2)?;
        }
        write!(f, "\n")?;

        // Write rows (per player)
        for i in 0..self.players.len() {
            write!(f, " -{} {:width$}", self.players[i].name(), "", width=name_length-self.players[i].name().len())?;
            write!(f, "{:>+width$.prec$} ", aggregate_preformance[i], width=2, prec=3)?;
            for j in 0..self.table_size {
                let val = self.avg_win[i][j];
                let mut base_string: ColoredString = ColoredString::from(format!("{:>+width$.prec$} ", val, width=2, prec=3).as_str());
                if i == best_preformer[j] {
                    base_string = base_string.on_green();
                } else if i == worst_preformer[j] {
                    base_string = base_string.on_red();
                }
                if f32::abs(val) < 0.002 {
                    base_string = base_string.yellow();
                } else if val > 0.0 {
                    base_string = base_string.green();
                } else {
                    base_string = base_string.red();
                }
                write!(f, "{}", base_string)?;
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
}


fn full_test<'a>(players: &'a [Box<dyn ActionSupplier>], table_size: usize, games: usize) -> FullTestResults<'a> {

    let mut accumulated_outcome: Vec<[f32; 16]> = vec![[0.0; 16]; players.len()];
    let mut game_count: Vec<[u64; 16]> = vec![[0; 16]; players.len()];

    let start = Instant::now();
    let mut rng = thread_rng();
    for _ in 0..games/100 {
        let mut player_order: Vec<usize> = (0..players.len()).collect();
        player_order.shuffle(&mut rng);

        for _ in 0..100 {
            let init_chips =vec![100.0; table_size];
            for i in 0..table_size {
                accumulated_outcome[player_order[i]][i] -= init_chips[i];
            }
            let mut rs = RoundState::new(init_chips);
            sim_round(&mut rs, players, &player_order);
            for i in 0..table_size {
                accumulated_outcome[player_order[i]][i] += rs.free_chips[i];
                game_count[player_order[i]][i] += 1;
            }
        }
    }
    let duration = start.elapsed();

    for i in 0..accumulated_outcome.len() {
        for j in 0..accumulated_outcome[i].len() {
            accumulated_outcome[i][j] /= game_count[i][j] as f32;
        }
    }

    FullTestResults {
        players: players,
        avg_win: accumulated_outcome,
        games_played: game_count,
        table_size: table_size,
        elapsed: duration,
    }
}

fn sim_round(rs: &mut RoundState, players: &[Box<dyn ActionSupplier>], player_order: &[usize]) {
    while !rs.is_finished() {
        let action_supplier = players[player_order[rs.turn as usize]].as_ref();
        let bet_size = action_supplier.get_action(rs);
        rs.do_action(bet_size);
    }
    rs.finish_game();
}
