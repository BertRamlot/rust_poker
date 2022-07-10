use colored::{Colorize, ColoredString};
use rand::{prelude::SliceRandom, thread_rng};
use std::time::{Duration, Instant};

use poker::round_state::RoundState;


pub trait ActionSupplier {
    fn get_action(&mut self, rs: &RoundState) -> f32;
    fn inform_finish(&mut self, rs: &RoundState, self_index: usize);
    fn name(&self) -> String;
}

pub struct BenchmarkResult {
    players: Vec<String>,
    games_played: Vec<[u64; 16]>,
    avg_win: Vec<[f32; 16]>, // avg_win[i][j] -> avg big_blind win for [PLAYER i] playing in [POSITION j]
    table_size: usize,
    pub elapsed: Duration,
}
impl<'a> std::fmt::Display for BenchmarkResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name_length = self.players.iter().map(|p| p.len()).max().unwrap();

        // Calculate total games played
        let total_games_played: u64 = self.games_played.iter().map(|a| a.iter().sum::<u64>()).sum::<u64>()/(self.table_size as u64);
        write!(f, "Simulated {} games in {:.2}s ({:.2} games per second)\n",
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
            aggregate_preformance[i] /= self.table_size as f32;
        }

        // Find best and worst preformers per position
        let mut best_preformer: Vec<usize> = Vec::new();
        let mut worst_preformer: Vec<usize> = Vec::new();
        for item_index in 0..1+self.table_size {
            let mut best_index: usize = 0;
            let mut worst_index: usize = 0;
            for i in 0..self.players.len() {
                if item_index == 0 {
                    if aggregate_preformance[i] > aggregate_preformance[best_index] {
                        best_index = i;
                    }
                    if aggregate_preformance[i] <= aggregate_preformance[worst_index] {
                        worst_index = i;
                    }
                } else {
                    let j: usize = item_index-1;
                    if self.avg_win[i][j] > self.avg_win[best_index][j] {
                        best_index = i;
                    }
                    if self.avg_win[i][j] <= self.avg_win[worst_index][j] {
                        worst_index = i;
                    }
                }
            }
            best_preformer.push(best_index);
            worst_preformer.push(worst_index);
        }

        // Write header
        const HEADER_TITLES_2_PLAYERS: [&str; 2] = ["SB (F0)", "BB (F1)"];
        const HEADER_TITLES_3_PLAYERS: [&str; 3] = ["BTN (F0)", "SB (F1)", "BB"];
        const HEADER_TITLES_DEFUALT: [&str; 4] = ["BTN", "SB (F0)", "BB", "P3 (F1)"];
        write!(f, "  {:^width$} ", "Player", width=name_length)?;
        write!(f, "{:^width$}|", "TOTAL", width=9)?;
        for i in 0..self.table_size {
            let header_item: String = match self.table_size {
                2 => HEADER_TITLES_2_PLAYERS[i].to_string(),
                3 => HEADER_TITLES_3_PLAYERS[i].to_string(),
                _ => {
                    if i < HEADER_TITLES_DEFUALT.len() {
                        HEADER_TITLES_DEFUALT[i].to_owned()
                    } else {
                        format!("P{}", i)
                    }
                }
            };

            write!(f, " {:<width$}", header_item, width=7)?;
        }
        write!(f, "\n")?;

        // Write rows (per player)
        for i in 0..self.players.len() {
            write!(f, " -{} {:width$}", self.players[i], "", width=name_length-self.players[i].len())?;
            for item_index in 0..1+self.table_size {
                let val: f32;
                if item_index == 0 {
                    val = aggregate_preformance[i];
                } else {
                    val = self.avg_win[i][item_index-1 as usize];    
                }
                let mut base_string: ColoredString = ColoredString::from(format!("{:>+width$.prec$} ", val, width=2, prec=3).as_str());
                if i == best_preformer[item_index] {
                    base_string = base_string.on_green();
                } else if i == worst_preformer[item_index] {
                    base_string = base_string.on_red();
                }
                if f32::abs(val) < 0.002 {
                    base_string = base_string.yellow();
                } else if val > 0.0 {
                    base_string = base_string.green();
                } else {
                    base_string = base_string.red();
                }
                write!(f, " {}", base_string)?;
                if item_index == 0 {
                    write!(f, " |")?;
                }
            }
            write!(f, "\n")?;
        }
        write!(f, "")
    }
}


pub fn benchmark_players<'a>(all_players: &mut [Box<dyn ActionSupplier + 'a>], active_player_indices: &[usize], table_size: usize, games: usize) -> BenchmarkResult {
    let player_count = active_player_indices.len();

    let mut accumulated_outcome: Vec<[f32; 16]> = vec![[0.0; 16]; player_count];
    let mut game_count: Vec<[u64; 16]> = vec![[0; 16]; player_count];

    let start = Instant::now();
    let mut rng = thread_rng();
    for _ in 0..games/100 {
        let mut player_order: Vec<usize> = (0..player_count).collect();
        player_order.shuffle(&mut rng);

        for _ in 0..100 {
            let init_chips =vec![70.0; table_size];
            for i in 0..table_size {
                accumulated_outcome[player_order[i]][i] -= init_chips[i];
            }
            let mut rs = RoundState::new(init_chips);
            // --- Simulate a round ---
            while !rs.is_finished() {
                let current_player_index = active_player_indices[player_order[rs.turn as usize]];
                let action_supplier = all_players[current_player_index].as_mut();
                let bet_size = action_supplier.get_action(&mut rs);
                rs.do_action(bet_size);
            }
            rs.finish_game();
        
            for i in 0..table_size {
                all_players[active_player_indices[player_order[i]]].inform_finish(&mut rs, i);
            }
            // --- End Simulate a round ---
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

    let player_names: Vec<String> = active_player_indices.iter().map(|i| all_players[*i].name().to_owned()).collect();
    BenchmarkResult {
        players: player_names,
        avg_win: accumulated_outcome,
        games_played: game_count,
        table_size,
        elapsed: duration,
    }
}

/*
fn sim_round<'a>(rs: &mut RoundState, players: &'a [Box<&'a mut dyn ActionSupplier<'a>>], player_order: &[usize]) {

}
*/
