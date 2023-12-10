# Rust poker

Rust library for poker (Texas hold 'em), includes:

- n card hand canonicalization
- 7 card hand evaluation
- Full round logic

## Performance

- 7 card hand canonicalization: ~ 2.5 M hands/second
- 7 card hand evaluations: ~ 1.4M hands/second
- 2 player game (w/ ~8 actions/game): ~ 250k games/second (= ~ 2.0M actions/second)
- 8 player game (w/ ~34 actions/game): ~  75k games/second (= ~ 2.5M actions/second)

Tested using a i7-9750H. Also, see the [benchmarks](./benches).

Note: Canonicalization incurs no extra overhead when also evaluating as this is part of the evaluation process.

## Usage

CardSet, Canonicalize, and Eval:

```rust
let card_set: CardSet = "As Ks 5h 6c 8c".into();
println!("{}", card_set);
// Outputs: As Ks 5h 6c 8c

let canonical_card_set: CardSet = card_set.as_canonical();
println!("{}", canonical_card_set);
// Outputs: As Ks 8h 6h 5d
// (You could also use the in-place variant: card_set.canonicalize())

let eval: i32 = canonical_card_set.evaluate();
println!("{}", eval);
// Outputs: 154030
// Note: evaluate() expects a canonical card set
```

RoundState:

```rust
// 2 players, player 1 has 10 BB worth of chips, player 2 has 20 BB worth of chips
let free_chips = vec![10.0f32, 20.0];
let mut rs = RoundState::new(free_chips);

while !rs.is_finished() {
    let betsize: f32 = 0.0;
    // Decision logic to determine the betsize:
    // - Fold:          betsize <  0
    // - Check(-raise): 0 <= bet_size <= check_amount
    // - Raise/All-in:  bet_size > check_amount
    
    rs.do_action(betsize);
}
```

## Testing

- Amount of different isomorphic hands is tested.
- Occurrence of eval types (Flush, straight, ...) is tested.
- Eval order is tested for a select amount of hands (see [eval_order_test.txt](./test_resources/eval_order_test.txt))
