use std::time::Duration;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use poker::card_set::CardSet;
use rand::{prelude::SliceRandom, SeedableRng, rngs::StdRng};

fn canonicalize_cardsets(card_sets: &mut [CardSet]) {
    for card_set in card_sets {
        card_set.canonicalize();
    }
}

fn eval_cardsets(card_sets: &mut [CardSet]) {
    for cardset in card_sets {
        cardset.canonicalize();
        cardset.evaluate();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng: StdRng = SeedableRng::seed_from_u64(0u64);

    let mut cardsets: Vec<CardSet> = vec![];
    for _ in 0..100_000 {
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rng);
        cardsets.push(CardSet::from(&deck[0..7]));
    }

    c.bench_function(
        "100k random canonicalizations",
        |b| b.iter(|| canonicalize_cardsets(black_box(&mut cardsets)))
    );

    c.bench_function(
        "100k random hand evaluations",
        |b| b.iter(|| eval_cardsets(black_box(&mut cardsets)))
    );
}

criterion_group!{
    name = benches;
    config = Criterion::default().measurement_time(Duration::from_secs(10));
    targets = criterion_benchmark
}
criterion_main!(benches);