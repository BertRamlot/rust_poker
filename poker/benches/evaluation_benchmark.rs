use criterion::{black_box, criterion_group, criterion_main, Criterion};

use poker::card_set::CardSet;
use rand::{prelude::SliceRandom, thread_rng};


fn eval_cardsets(cardsets: &[CardSet]) {
    for cardset in cardsets {
        let mut cs = cardset.clone();
        cs.canonicalize();
        cs.evaluate();
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = thread_rng();

    let mut cardsets: Vec<CardSet> = vec![];
    for _ in 0..100_000 {
        let mut deck: Vec<u8> = (0..52).collect();
        deck.shuffle(&mut rng);
        cardsets.push(CardSet::from(&deck[0..7]));
    }

    c.bench_function(
        "rnd_cardset_eval",
        |b| b.iter(|| eval_cardsets(black_box(&cardsets)))
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);