
#[cfg(test)]
mod cardset_test {
    use std::{collections::HashSet, fs::File, io::{BufReader, BufRead}, path::Path};

    use poker::card_set::CardSet;

	fn increment_cardset(card_set: &mut CardSet) -> bool {
		let card_count = card_set.len();
        card_set.cards[card_count-1].0 += 1;
        for i in (0..card_count).rev() {
            if card_set.cards[i].0 as usize >= 52-(card_count-1-i) {
                if i == 0 {
                    // Max value was exceeded, failed to increment
                    return false;
                }
                card_set.cards[i-1].0 += 1;
                continue;
            }
            // Found a value who's max is not exceeded
            for j in i+1..card_count {
                card_set.cards[j].0 = card_set.cards[i].0 + (j - i) as u8;
            }
            break;
        }
		return true;
	}

	const EXPECTED_HAND_COUNT: [u64; 8] = [0, 52, 1326, 22100, 270725, 2598960, 20358520, 133784560];
	const EXPECTED_ISOMORPHIC_HAND_COUNT: [u32; 8] = [0, 13, 169, 1755, 16432, 134459, 962988, 6009159];

	#[test]
	fn test_count_different_isomorphic_hands() {
        for card_count in 1..EXPECTED_ISOMORPHIC_HAND_COUNT.len() {
			let mut hand_count = 0u64;
            let mut card_set: CardSet = (0..card_count as u8).collect::<Vec<u8>>().into();
            let mut seen_identifiers = HashSet::new();
            loop {
				hand_count += 1;
                let cannonical = card_set.clone().as_canonical();
                seen_identifiers.insert(cannonical.identifier());
                if !increment_cardset(&mut card_set) {
                    break;
                }
            }
			assert_eq!(
				EXPECTED_HAND_COUNT[card_count],
				hand_count, 
				"Incorrect amount of hands for CarSet with {} cards",
				card_count
			);
            assert_eq!(
				EXPECTED_ISOMORPHIC_HAND_COUNT[card_count],
				seen_identifiers.len() as u32,
				"Incorrect amount of isomorphic hands for CarSet with {} cards",
				card_count
			);
		}
	}

	#[test]
	fn test_canonical_eval() {
		const EXPECTED_EVAL_COUNTS: [(&str, u32); 9] = [
			("High card",       23294460),
			("One pair",        58627800),
			("Two pair",        31433400),
			("Three of a kind",  6461620),
			("Straight",         6180020),
			("Flush",            4047644),
			("Full house",       3473184),
			("Four of a kind",    224848),
			("St/Ro flush",        41584),
		];

		let mut card_set: CardSet = (0..7u8).collect::<Vec<u8>>().into();
		let mut eval_type_count = [0u32; 9];
		let mut hand_count = 0u64;
		loop {
            let cannonical = card_set.clone().as_canonical();
            let eval_type = cannonical.evaluate() >> 20;
			eval_type_count[eval_type as usize] += 1;
			hand_count += 1;
			if !increment_cardset(&mut card_set) {
				break;
			}
		}

		assert_eq!(
			EXPECTED_HAND_COUNT[7],
			hand_count,
			"Incorrect amount of hands for CarSet with 7 cards"
		);

		let mut success = true;
		let mut err_string = format!("        {:15} {:8}    {:8}\n", "Hand type", "Expected", "Got");
		for i in 0..eval_type_count.len() {
			let (type_str, count) = EXPECTED_EVAL_COUNTS[i];
			if count == eval_type_count[i] {
				err_string.push_str(&format!("        {:15} {:8} == {:8}\n", type_str, count, eval_type_count[i]));
			} else {
				err_string.push_str(&format!("Wrong > {:15} {:8} != {:8}\n", type_str, count, eval_type_count[i]));
				success = false;
			}
		}
		assert!(
			success,
			"{}",
			err_string
		);
	}

	#[test]
	fn test_eval_order() {
		let path = Path::new("./test_resources/eval_order_test.txt");
		let file = match File::open(path) {
			Ok(f) => f,
			Err(e) => panic!("Failed to eval_order_test file: {}", e),
		};
		let reader = BufReader::new(file);
	
		let mut section = "".to_owned();
		let mut prev_str = "".to_owned();
		let mut prev_eval = -1;
		for (i, line) in reader.lines().enumerate() {
            let curr_str = line.unwrap();
			if curr_str.len() == 0 {
				continue;
			}
			if curr_str.chars().nth(0).unwrap() == '#' {
				section = curr_str[2..].to_owned();
				continue;
			}
			let cards: CardSet = curr_str[2..].into();
			let curr_eval = cards.as_canonical().evaluate();

			if prev_str != "" {
				let success = match curr_str.chars().nth(0).unwrap() {
					'>' => prev_eval > curr_eval,
					'=' => prev_eval == curr_eval,
					'<' => prev_eval < curr_eval,
					_ => panic!("Invalid comperator"),
				};
				assert!(
					success,
					"\n\t<Line={}, Section='{}'> Comparison does not hold:\n\t\tPrevious: '{}' -> Eval={}\n\t\tCurrent : '{}' -> Eval={}\n",
					i+1, section,
					prev_str, prev_eval,
					curr_str, curr_eval
				);
			}

			prev_eval = curr_eval;
			prev_str = curr_str;
		}
	}
}