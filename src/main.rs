#[macro_use] extern crate itertools;


use crate::poker::convertor::{CardSet};

mod poker;



fn main() {
    println!("Hello, world!");
    let cards: CardSet = vec![1, 2, 40, 4, 5].into();
    println!("Cards {}", cards);
	let can_cards = cards.as_canonical();
	println!("CanonicalCards {}", can_cards);
}

