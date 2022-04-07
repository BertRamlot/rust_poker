use std::{fmt::{self, Write}};

use itertools::Itertools;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card(pub u8);

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardSet(pub Vec<Card>);


impl fmt::Display for Card {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
		const NRS: &str = "23456789TJQKA";
        const SUITS: &str = "cdhs";
	
        fmt.write_char(NRS.chars().nth((self.0 % 13) as usize).unwrap())?;
        fmt.write_char(SUITS.chars().nth((self.0 / 13) as usize).unwrap())?;

        Ok(())
    }
}

impl fmt::Display for CardSet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for (i, card) in self.0.iter().enumerate() {
            fmt.write_fmt(format_args!("{}", card))?;
            if i != self.0.len()-1 {
                fmt.write_char(' ')?;
            }
        }
        Ok(())
    }
}

impl From<u8> for Card {
    fn from(c: u8) -> Self {
        Card(c)
    }
}

impl From<Vec<u8>> for CardSet {
    fn from(c: Vec<u8>) -> Self {
        CardSet(c.iter().map(|&x| x.into()).collect())
    }
}

impl Into<CardSet> for &str {
	fn into(self) -> CardSet {
        // Supported formats:
        // "4h.2c.3c.As.9s.Qs" with '.' as any character
		const NRS: &str = "23456789TJQKA";
        const SUITS: &str = "cdhs";

		let mut cards = Vec::new();

		for i in 0..self.chars().count()/3 {
            let nr_char = NRS.chars().nth(3*i).unwrap();
            let suit_char = SUITS.chars().nth(3*i+1).unwrap();
            let suit = self.chars().position(|c| c == nr_char).unwrap();
            let nr = self.chars().position(|c| c == suit_char).unwrap();
			cards.push(Card((nr * 13 + suit) as u8));
		}

		CardSet(cards)
	}
}

impl CardSet {
    pub fn increment(&mut self) -> bool {
        let card_count = self.0.len();
        self.0[card_count-1].0 += 1;
        for i in (0..card_count).rev() {
            if self.0[i].0 as usize >= 52-(card_count-1-i) {
                if i == 0 {
                    // Max value was exceeded, failed to increment
                    return false;
                }
                self.0[i-1].0 += 1;
                continue;
            }
            // Found a value who's max is was not exceeded
            for j in i+1..card_count {
                self.0[j].0 = self.0[i].0 + (j - i) as u8;
            }
            break;            
        }
		return true;
	}

    pub fn identifier(&self) -> u64 {
        let mut id = 0u64;
        for c in self.0.iter() {
            id |= c.0 as u64;
            id <<= 8;
        }
        id
    }

    pub fn as_canonical(mut self) -> Self {
        self.canonicalize();
        self
    }

    pub fn canonicalize(&mut self) {
        const PRIMES: [usize; 13] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41];

        let nrs = self.0.iter().map(|card| (card.0 % 13) as usize).collect::<Vec<usize>>();
        let suits = self.0.iter().map(|card| (card.0 / 13) as usize).collect::<Vec<usize>>();
        let mut suit_count: [usize; 4] = [0, 0, 0, 0];
        for &s in &suits {
            suit_count[s] += 1;
        }

        // Determine suit mapping
        let mut suit_second_ranking: [usize; 4] = [1, 1, 1, 1];
        for (&nr, &suit) in itertools::izip!(&nrs, &suits) {
            suit_second_ranking[suit] *= PRIMES[nr];
        }

        let inv_suit_mapping: Vec<usize> = (0..4).into_iter().enumerate()
            .sorted_by_key(|&(i, _)| suit_second_ranking[i])
            .sorted_by_key(|&(i, _)| suit_count[i])
            .map(|(_, i)| i).collect::<Vec<usize>>();

        let mut suit_mapping: [usize; 4] = [0, 0, 0, 0];
        for i in 0..4 {
            suit_mapping[inv_suit_mapping[i]] = (3 - i) as usize;
        }

        let out = itertools::izip!(&nrs, &suits)
            .map(|(&nr, &suit)| suit_mapping[suit]*13 + nr)
            .sorted()
            .map(|c| Card(c as u8))
            .collect::<Vec<Card>>();

        self.0 = out;
    }

    pub fn evaluate(&self) -> u32 {
        unimplemented!()
    }
}
/*
impl CanonicalCardSet {
    pub fn canonical_cards_to_eval(&self) -> u32 {
        let HIGH_CARD_START : u32 = 0 * (1 << 20) + 1; // 0;
        let PAIR_START : u32 = 1 * (1 << 20); // HIGH_CARD_START + 154440;
        let TWO_PAIR_START : u32 = 2 * (1 << 20); // PAIR_START + 22308;
        let THREE_OF_KIND_START : u32 = 3 * (1 << 20); // TWO_PAIR_START + 1870;
        let STRAIGHT_START : u32 = 4 * (1 << 20); // THREE_OF_KIND_START + 2028;
        let FLUSH_START : u32 = 5 * (1 << 20); // STRAIGHT_START + 10;
        let FULL_HOUSE_START : u32 = 6 * (1 << 20); // FLUSH_START + 154440;
        let FOUR_OF_KIND_START : u32 = 7 * (1 << 20); // FULL_HOUSE_START + 181;
        let STRAIGHT_FLUSH_START : u32 = 8 * (1 << 20); // FOUR_OF_KIND_START + 181;


        let nrs = canonical_cards.into_iter().map(|&card| card % 13).collect::<Vec<Card>>();
        let is_flush = canonical_cards[4] / 13 == 3;
        if is_flush {
            let straight = -1;
            let current_straight_count = 1;
            for i in 1..nrs.len() {
                if (canonical_cards[i] / 13 != 3) break;
                if (nrs[i - 1] != nrs[i] + 1)
                {
                    current_straight_count = 1;
                    continue;
                }
                current_straight_count++;
                if (nrs[i] == 0 && nrs[0] == 12) {
                    current_straight_count++;
                }
                if (current_straight_count >= 5) {
                    // (Highest) Straight found
                    straight = nrs[i];
                    break;
                }
            }

            // Full House and 4 of a kind are not possible because there is a flush
            if (straight != -1) {
                // Royal/Straight flush
                return STRAIGHT_FLUSH_START + straight;
            } else {
                // Flush
                return FLUSH_START + 11880 * nrs[0] + 990 * nrs[1] + 90 * nrs[2] + 9 * nrs[3] + nrs[4];
            }
        }
        else
        {
            nrs.sort();
            // std::sort(nrs.begin(), nrs.end(), std::greater<>());

            // Straight
            let straight = -1;
            unsigned char current_straight_count = 1;
            for (size_t i = 1; i < nrs.size(); i++)
            {
                if (nrs[i - 1] != nrs[i] + 1)
                {
                    if (nrs[i - 1] != nrs[i])
                    {
                        current_straight_count = 1;
                    }
                    continue;
                }
                current_straight_count++;
                if (nrs[i] == 0 && nrs[0] == 12)
                {
                    current_straight_count++;
                }
                if (current_straight_count >= 5)
                {
                    // (Highest) Straight found
                    straight = nrs[i];
                    break;
                }
            }


            // Kinds
            let four_kind = -1;
            let three_kind = -1;
            let two_kind_h = -1;
            let two_kind_l = -1;

            let kind_count: u8 = 1;
            for (size_t i = 1; i < nrs.size(); i++)
            {
                if (nrs[i - 1] == nrs[i]) {
                    kind_count++;
                    if (i != nrs.size() - 1) continue;
                }
                if (four_kind == -1 && kind_count == 4) {
                    four_kind = nrs[i-1];
                    break;
                }
                else if (three_kind == -1 && kind_count == 3) {
                    three_kind = nrs[i-1];
                }
                else if (two_kind_h == -1 && kind_count >= 2) {
                    two_kind_h = nrs[i-1];
                }
                else if (two_kind_l == -1 && kind_count == 2) {
                    two_kind_l = nrs[i-1];
                }
                kind_count = 1;
            }

            if (four_kind != -1)
            {
                // Four of a kind
                return FOUR_OF_KIND_START + 13 * four_kind + (nrs[0] == four_kind ? nrs[4] : nrs[0]);
            }
            else if (three_kind != -1 && two_kind_h != -1)
            {
                // Full house
                return FULL_HOUSE_START + 13 * three_kind + two_kind_h;
            }
            else if (straight != -1)
            {
                // Straight
                return STRAIGHT_START + straight;
            }
            else if (three_kind != -1 && two_kind_h == -1)
            {
                // Three of a kind
                uint32_t kicker_0 = 65535;
                uint32_t kicker_1 = 65535;
                for (int i = 0; i < 5; i++)
                {
                    if (nrs[i] == three_kind)
                    {
                        i+=2; // Go faster
                        continue;
                    }
                    if (kicker_0 == 65535) {
                        kicker_0 = nrs[i];
                    }
                    else {
                        kicker_1 = nrs[i];
                        break;
                    }
                }
                return THREE_OF_KIND_START + 156 * three_kind + 12 * kicker_0 + kicker_1;
            } else if (two_kind_h != -1) {
                if (two_kind_l != -1) {
                    // Two pair
                    uint32_t kicker = 65535;
                    for (int i = 0; i < 5; i++) {
                        if (nrs[i] == two_kind_h || nrs[i] == two_kind_l) {
                            i++; // Go faster
                            continue;
                        }
                        kicker = nrs[i];
                        break;
                    }
                    return TWO_PAIR_START + 156 * (two_kind_h - 1) + 13 * two_kind_l + kicker;
                } else {
                    // Pair
                    uint32_t kicker_0 = 65535;
                    uint32_t kicker_1 = 65535;
                    uint32_t kicker_2 = 65535;
                    for (int i = 0; i < 5; i++)
                    {
                        if (nrs[i] == two_kind_h) {
                            i++; // Go faster
                            continue;
                        }
                        if (kicker_0 == 65535) {
                            kicker_0 = nrs[i];
                        } else if (kicker_1 == 65535) {
                            kicker_1 = nrs[i];
                        } else {
                            kicker_2 = nrs[i];
                            break;
                        }
                    }
                    return PAIR_START + 1716 * two_kind_h + 132 * kicker_0 + 11 * kicker_1 + kicker_2;
                }
            } else {
                // High card
                return HIGH_CARD_START + 11880 * nrs[0] + 990 * nrs[1] + 90 * nrs[2] + 9 * nrs[3] + nrs[4];
            }
        }
    }
}
    */