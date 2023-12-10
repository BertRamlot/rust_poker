use criterion::{black_box, criterion_group, criterion_main, Criterion};

use poker::round_state::RoundState;
use rand::{SeedableRng, rngs::StdRng, Rng};

fn simulate_rounds(player_count: usize, init_stack: f32, bet_size: f32, itts: i32) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(0u64);

    for _ in 0..itts {
        let mut rs = RoundState::new(vec![init_stack; player_count]);

        while !rs.is_finished() {
            let f: f32 = rng.gen();
            if f < 0.05 {
                // fold
                rs.do_action(-1.0);
            } else if f < 0.8 {
                // check
                rs.do_action(0.0);
            } else {
                // raise
                rs.do_action(bet_size);
            }
        }
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    
    c.bench_function("1k rounds (2 players, ~8 actions)", |b| b.iter(|| simulate_rounds(
        black_box(2),
        black_box(100.0),
        black_box(10.0),
        black_box(1_000)
    )));

    c.bench_function("1k rounds (8 players, ~34 actions)", |b| b.iter(|| simulate_rounds(
        black_box(8),
        black_box(100.0),
        black_box(10.0),
        black_box(1_000)
    )));
}

criterion_group!{
    name = benches;
    config = Criterion::default();
    targets = criterion_benchmark
}
criterion_main!(benches);