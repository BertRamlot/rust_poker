#[macro_use] extern crate itertools;


use crate::poker::convertor::{CardSet};

mod poker;



fn main() {
    println!("Hello, world!");
    let cards: CardSet = vec![12, 11, 10, 50, 51, 32, 20].into();
    println!("Cards: '{}'", cards);
    println!("Cards: '{:?}'", cards);
	let can_cards = cards.as_canonical();
	println!("Canonical: '{}'", can_cards);
	println!("Canonical: '{:?}'", can_cards);

}

