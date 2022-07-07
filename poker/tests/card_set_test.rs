
#[cfg(test)]
mod cardset_test {
    use std::{collections::HashSet, fs::File, io::{BufReader, BufRead}, path::Path};

    use poker::card::CardSet;

	const EXPECTED_HAND_COUNT: [u64; 8] = [0, 52, 1326, 22100, 270725, 2598960, 20358520, 133784560];

	#[test]
	fn test_count_different_isomorphic_hands() {
		const EXPECTED_COUNTS: [u32; 8] = [0, 13, 169, 1755, 16432, 134459, 962988, 6009159];


        for card_count in 1..EXPECTED_COUNTS.len() {
			let mut hand_count = 0u64;
            // println!("Starting card_count={}", card_count);
            let mut card_vec = Vec::new();
            for i in 0..card_count {
                card_vec.push(i as u8);
            }
            let mut cards: CardSet = card_vec.into();
            let mut seen_canonicals = HashSet::new();
            loop {
                let cannonical = cards.clone().as_canonical();
                seen_canonicals.insert(cannonical.identifier());
				hand_count += 1;
                if !cards.increment() {
                    break;
                }
            }
			assert_eq!(EXPECTED_HAND_COUNT[card_count], hand_count, 
				"Expected {} different generated card_sets with {} cards, got {}", EXPECTED_HAND_COUNT[card_count], card_count, hand_count);
            assert_eq!(EXPECTED_COUNTS[card_count], seen_canonicals.len() as u32,
				"Expected {} different isomorphic card_sets with {} cards, got {}", EXPECTED_COUNTS[card_count], card_count, seen_canonicals.len());
		}
	}

	#[test]
	fn test_canonical_eval() {
		const EXPECTED_COUNTS: [(&str, u32); 9] = [
			("St/Ro flush",        41584),
			("Four of a kind",    224848),
			("Full house",       3473184),
			("Flush",            4047644),
			("Straight",         6180020),
			("Three of a kind",  6461620),
			("Two pair",        31433400),
			("One pair",        58627800),
			("High card",       23294460)
		];

		let mut input: CardSet = vec![0, 1, 2, 3, 4, 5, 6].into();
		let mut eval_type_count = vec![0u32; 9];
		let mut hand_count = 0u64;
		loop {
            let output = input.clone().as_canonical();
            let eval = output.evaluate();
			let eval_type_count_index = eval_type_count.len()-1 - ((eval >> 20) as usize);
			eval_type_count[eval_type_count_index] += 1;
			hand_count += 1;
			if !input.increment() {
				break;
			}
		}

		assert_eq!(EXPECTED_HAND_COUNT[7], hand_count,
			"Expected {} different generated card_sets with {} cards, got {}", EXPECTED_HAND_COUNT[7], 7, hand_count);

		let mut success = true;
		for i in 0..eval_type_count.len() {
			let (_s, c) = EXPECTED_COUNTS[i];
			if c != eval_type_count[i] {
				success = false;
				break;
			}
		}

		if !success {
			let mut err_string = format!("\n \t{:15} {:8}    {:8}\n", "Hand type", "Expected", "Got");
			for i in 0..eval_type_count.len() {
				let (s, c) = EXPECTED_COUNTS[i];
				if c == eval_type_count[i] {
					err_string.push_str(&format!(" \t{:15}: {:8} == {:8}\n", s, c, eval_type_count[i]));
				} else {
					err_string.push_str(&format!(">\t{:15}: {:8} != {:8}\n", s, c, eval_type_count[i]));
				}
			}
			panic!("{}", err_string);
		}
	}

	#[test]
	fn test_eval_order() {
		let path = Path::new("./test_resources/eval_order_test.txt");
		let file = match File::open(path) {
			Ok(f) => f,
			Err(e) => panic!("Failed to open file: {}", e),
		};
		let reader = BufReader::new(file);
	
		let mut last_comment = "???".to_owned();
		let mut prev_str = "None".to_owned();
		let mut prev_eval = 1;
		for (i, line) in reader.lines().enumerate() {
            let curr_str = line.unwrap();
			if curr_str.len() == 0 {
				continue;
			}
			if curr_str.chars().nth(0).unwrap() == '#' {
				last_comment = (&curr_str[2..]).to_owned();
				continue;
			}
			let cards: CardSet = (&curr_str[2..]).into();
			let canonical_cards = cards.as_canonical();
			let curr_eval = canonical_cards.evaluate();
			if prev_str != "None" {
				let success = match curr_str.chars().nth(0).unwrap() {
					'>' => prev_eval > curr_eval,
					'=' => prev_eval == curr_eval,
					_ => panic!("Invalid eval order test file"),
				};
				assert!(
					success,
					"\n\t<Line={}, Section='{}'> Expected different order\n\t\tPrevious: '{}' -> Eval={}\n\t\tCurrent : '{}' -> Eval={}\n",
					i+1, last_comment, prev_str, prev_eval, curr_str, curr_eval
				);
			}
			prev_eval = curr_eval;
			prev_str = curr_str;
		}
	}
}