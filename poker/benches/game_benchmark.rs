use criterion::{black_box, criterion_group, criterion_main, Criterion};

use poker::round_state::RoundState;

fn full_game(player_count: usize, init_stack: f32, bet_size: f32) {
    let mut rs = RoundState::new(vec![init_stack; player_count]);
    // println!("{}", rs);
    while !rs.is_finished() {
        rs.do_action(bet_size);
    }
    rs.finish_game();
}

fn criterion_benchmark(c: &mut Criterion) {
    
    c.bench_function("full_game 2", |b| b.iter(|| full_game(
        black_box(2),
        black_box(100.0),
        black_box(10.0),
    )));

    c.bench_function("full_game 8", |b| b.iter(|| full_game(
        black_box(8),
        black_box(100.0),
        black_box(10.0),
    )));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);