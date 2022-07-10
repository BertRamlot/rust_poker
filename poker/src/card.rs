use std::fmt::{self, Write};
use std::slice::{Iter, IterMut};
use itertools::Itertools;


// Card
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card(
    pub u8
);

const NRS: &str = "23456789TJQKA";
const SUITS: &str = "cdhs";

impl fmt::Display for Card {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_char(NRS.chars().nth((self.0 % 13) as usize).unwrap())?;
        fmt.write_char(SUITS.chars().nth((self.0 / 13) as usize).unwrap())?;

        Ok(())
    }
}

impl From<u8> for Card {
    fn from(c: u8) -> Self {
        Card(c)
    }
}

impl From<&str> for Card {
    fn from(s: &str) -> Self {
        let nr_char = s.chars().nth(0).unwrap();
        let suit_char = s.chars().nth(1).unwrap();
        let nr = NRS.chars().position(|c| c == nr_char).unwrap();
        let suit = SUITS.chars().position(|c| c == suit_char).unwrap();
        Card::from((suit * 13 + nr) as u8)
    }
}


// CardSet

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardSet {
    cards: [Card; 7],
    size: u8
}

impl fmt::Display for CardSet {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        for (i, card) in self.iter().enumerate() {
            fmt.write_fmt(format_args!("{}", card))?;
            if i != self.len()-1 {
                fmt.write_char(' ')?;
            }
        }
        Ok(())
    }
}

impl From<Vec<u8>> for CardSet {
    fn from(c: Vec<u8>) -> Self {
        let vec = c.iter().map(|&x| x.into()).collect::<Vec<Card>>();
        vec.into()
    }
}

impl From<Vec<Card>> for CardSet {
    fn from(cards: Vec<Card>) -> Self {
        CardSet::new(cards.as_slice())
    }
}

impl From<&[u8]> for CardSet {
    fn from(c: &[u8]) -> Self {
        let vec = c.iter().map(|&x| x.into()).collect::<Vec<Card>>();
        vec.into()
    }
}

impl From<&[Card]> for CardSet {
    fn from(c: &[Card]) -> Self {
        let vec = c.iter().map(|&x| x.into()).collect::<Vec<Card>>();
        vec.into()
    }
}

impl From<&str> for CardSet {
	fn from(s: &str) -> Self {
        // Supported formats:
        // "4h.2c.3c.As.9s.Qs" with '.' as any character
		let mut cards = Vec::new();
		for i in 0..(s.chars().count()+1)/3 {
            cards.push(Card::from(&s[i*3..(i*3+2)]));
		}
        cards.into()
	}
}

impl FromIterator<Card> for CardSet {
    fn from_iter<I: IntoIterator<Item = Card>>(iter: I) -> Self {
        let vec = iter.into_iter().collect::<Vec<Card>>();
        vec.into()
    }
}

impl CardSet {
    pub fn new(cards: &[Card]) -> Self {
        let mut cs = CardSet {
            cards: [255.into(); 7],
            size: std::cmp::min(cards.len() as u8, 7)
        };
        cs.update(cards);
        cs
    }

    fn update(&mut self, cards: &[Card]) {
        let length = std::cmp::min(cards.len(), self.cards.len());
        self.cards[0..length].copy_from_slice(cards[0..length].as_ref());
    }

    pub fn len(&self) -> usize {
        self.size as usize
    }

    pub fn iter(&self) -> Iter<'_, Card> {
        self.cards[..self.size as usize].iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Card> {
        self.cards[..self.size as usize].iter_mut()
    }

    pub fn increment(&mut self) -> bool {
        let card_count = self.len();
        self.cards[card_count-1].0 += 1;
        for i in (0..card_count).rev() {
            if self.cards[i].0 as usize >= 52-(card_count-1-i) {
                if i == 0 {
                    // Max value was exceeded, failed to increment
                    return false;
                }
                self.cards[i-1].0 += 1;
                continue;
            }
            // Found a value who's max is not exceeded
            for j in i+1..card_count {
                self.cards[j].0 = self.cards[i].0 + (j - i) as u8;
            }
            break;            
        }
		return true;
	}

    pub fn identifier(&self) -> u64 {
        let mut id = 0u64;
        for (i, c) in self.iter().enumerate() {
            id |= c.0 as u64;
            if i != self.len()-1 {
                id <<= 8;
            }
        }
        id
    }

    pub fn as_canonical(mut self) -> Self {
        self.canonicalize();
        self
    }

    // ~ 0.52 us
    pub fn canonicalize(&mut self) {
        const PRIMES: [u64; 13] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41];
        let nrs = {
            let mut cs = self.clone();
            for c in cs.iter_mut() {
                c.0 %= 13;
            }
            cs
        };
        
        let suits = {
            let mut cs = self.clone();
            for c in cs.iter_mut() {
                c.0 /= 13;
            }
            cs
        };

        let mut suit_count: [usize; 4] = [0, 0, 0, 0];
        for &s in suits.iter() {
            suit_count[s.0 as usize] += 1;
        }
        
        let mut suit_second_ranking: [u64; 4] = [1, 1, 1, 1];
        for (&nr, &suit) in itertools::izip!(nrs.iter(), suits.iter()) {
            suit_second_ranking[suit.0 as usize] *= PRIMES[nr.0 as usize];
        }
        

        let inv_suit_mapping: Vec<usize> = (0..4).into_iter().enumerate()
            .sorted_by_key(|&(i, _)| suit_second_ranking[i])
            .sorted_by_key(|&(i, _)| suit_count[i])
            .map(|(_, i)| i).collect::<Vec<usize>>();

        // println!("{:?} -> {:?}", suit_count, inv_suit_mapping);

        // Determine suit mapping
        let mut suit_mapping: [u8; 4] = [0, 0, 0, 0];
        for i in 0..4 {
            suit_mapping[inv_suit_mapping[i]] = i as u8;
        }

        let out = itertools::izip!(nrs.iter(), suits.iter())
            .map(|(&nr, &suit)| suit_mapping[suit.0 as usize]*13 + nr.0)
            .sorted_by_key(|&x| 255u8-x)
            .map(|c| c.into())
            .collect::<Vec<Card>>();

        self.update(out.as_slice());
    }

    // ~ 0.24 us
    pub fn evaluate(&self) -> i32 {
        // Expects the card vector to be canonicalized
        // Expects the card vector to be of length 7

        // -----------ALG------------
        // <Calculate 'Flush'>
        // if Flush:
        //   <Calculate 'Straight'>
        //   if Straight:
        //     return STRAIGHT_FLUSH_START (includes ROYAL_FLUSH)
        //   else:
        //     return FLUSH
        //
        // <Calculate all 'n-of-a-kinds'>
        // if 4-of-kind:
        //   return FOUR_OF_KIND (aka QUADS)
        // if 3-of-kind && 2-of-kind:
        //   return FULL_HOUSE
        // 
        // <Calculate 'Straight'>
        // if Straight:
        //   return STRAIGHT
        // 
        // if 3-of-kind:
        //   return THREE_OF_KIND
        // if 2-of-kind x2:
        //   return TWO_PAIR
        // if 2-of-kind:
        //   return PAIR
        // else:
        //   return HIGH_CARD
        // --------------------------

        const HIGH_CARD_START : i32      = 0 * (1 << 20) + 1; // 0;
        const PAIR_START : i32           = 1 * (1 << 20); // HIGH_CARD_START + 154440;
        const TWO_PAIR_START : i32       = 2 * (1 << 20); // PAIR_START + 22308;
        const THREE_OF_KIND_START : i32  = 3 * (1 << 20); // TWO_PAIR_START + 1870;
        const STRAIGHT_START : i32       = 4 * (1 << 20); // THREE_OF_KIND_START + 2028;
        const FLUSH_START : i32          = 5 * (1 << 20); // STRAIGHT_START + 10;
        const FULL_HOUSE_START : i32     = 6 * (1 << 20); // FLUSH_START + 154440;
        const FOUR_OF_KIND_START : i32   = 7 * (1 << 20); // FULL_HOUSE_START + 181;
        const STRAIGHT_FLUSH_START : i32 = 8 * (1 << 20); // FOUR_OF_KIND_START + 181;

        let mut nrs = {
            let mut cs = self.clone();
            for c in cs.iter_mut() {
                c.0 %= 13;
            }
            cs
        };


        let is_flush = self.cards[4].0 / 13 == 3;
        if is_flush {
            let mut straight = -1i32;
            let mut current_straight_count = 1;
            for i in 1..nrs.len() {
                if self.cards[i].0 / 13 != 3 {
                    break;
                }
                if nrs.cards[i-1].0 != nrs.cards[i].0 + 1 {
                    current_straight_count = 1;
                    continue;
                }
                current_straight_count += 1;
                if current_straight_count == 4 && nrs.cards[i].0 == 0 && nrs.cards[0].0 == 12 {
                    // Ace causes low straight
                    straight = 0;
                    break;
                }
                if current_straight_count == 5 {
                    // (Highest) Straight found
                    straight = 1 + (nrs.cards[i].0 as i32);
                    break;
                }
            }
            // Only 'Flush' or 'Straight Flush' or 'Royal Flush' are possible
            if straight != -1 {
                // Royal/Straight flush
                return STRAIGHT_FLUSH_START + (straight as i32);
            } else {
                // Flush
                return FLUSH_START + 
                    (nrs.cards[0].0 as i32) * 11880 + 
                    (nrs.cards[1].0 as i32) * 990 + 
                    (nrs.cards[2].0 as i32) * 90 + 
                    (nrs.cards[3].0 as i32) * 9 + 
                    (nrs.cards[4].0 as i32);
            }
        }
        // Not a flush => Not 'Flush' or 'Straight Flush' or 'Royal Flush'
        
        nrs.cards.sort_by(|a, b| b.0.cmp(&a.0));

        // n-of-a-kind
        let mut four_kind = -1;
        let mut three_kind = -1;
        let mut two_kind_h = -1;
        let mut two_kind_l = -1;

        let mut kind_count: u8 = 1;
        for i in 1..nrs.len() {
            if nrs.cards[i-1] == nrs.cards[i] {
                kind_count += 1;
                if i != nrs.len()-1 {
                    // We are not at the end of card_set && we didn't broke repeating pattern
                    // Wait until we are at the end or we break repeating pattern to take conclusions
                    continue;
                }
            }
            if four_kind == -1 && kind_count == 4 {
                four_kind = nrs.cards[i-1].0 as i32;
                break;
            } else if three_kind == -1 && kind_count == 3 {
                three_kind = nrs.cards[i-1].0 as i32;
            } else if two_kind_h == -1 && kind_count >= 2 {
                two_kind_h = nrs.cards[i-1].0 as i32;
            } else if two_kind_l == -1 && kind_count == 2 {
                two_kind_l = nrs.cards[i-1].0 as i32;
            }
            // We broke pattern
            kind_count = 1;
        }

        if four_kind != -1 {
            // Four of a kind
            let kicker_value: i32;
            if nrs.cards[0].0 as i32 == four_kind {
                kicker_value = nrs.cards[4].0 as i32;
            } else {
                kicker_value = nrs.cards[0].0 as i32;
            }
            return FOUR_OF_KIND_START + 13 * four_kind + kicker_value;
        } else if three_kind != -1 && two_kind_h != -1 {
            // Full house
            return FULL_HOUSE_START + 13 * three_kind + two_kind_h;
        }

        // Straight
        let mut straight: i32 = -1;
        let mut current_straight_count = 1;
        for i in 1..nrs.len() {
            if nrs.cards[i-1].0 != nrs.cards[i].0 + 1 {
                if nrs.cards[i-1].0 != nrs.cards[i].0 {
                    current_straight_count = 1;
                }
                continue;
            }
            current_straight_count += 1;
            if current_straight_count == 4 && nrs.cards[i].0 == 0 && nrs.cards[0].0 == 12  {
                // Ace causes low straight
                straight = 0;
                break;
            }
            if current_straight_count == 5 {
                // (Highest) Straight found
                straight = 1 + (nrs.cards[i].0 as i32);
                break;
            }
        }
        
        if straight != -1 {
            // Straight
            return STRAIGHT_START + straight;
        } else if three_kind != -1 && two_kind_h == -1 {
            // Three of a kind
            let mut kicker_0 = -1;
            let mut kicker_1 = -1;
            let mut i = 0;
            while i < 5 {
                if nrs.cards[i].0 as i32 == three_kind {
                    i += 3; // Go faster
                    continue;
                }
                if kicker_0 == -1 {
                    kicker_0 = nrs.cards[i].0 as i32;
                } else {
                    kicker_1 = nrs.cards[i].0 as i32;
                    break;
                }
                i += 1;
            }
            return THREE_OF_KIND_START + 156 * three_kind + 12 * kicker_0 + kicker_1;
        } else if two_kind_h != -1 {
            if two_kind_l != -1 {
                // Two pair
                let mut kicker: i32 = -1;
                let mut i = 0;
                while i < 5 {
                    if nrs.cards[i].0 == two_kind_h as u8 || nrs.cards[i].0 == two_kind_l as u8 {
                        i += 2; // Go faster
                        continue;
                    }
                    kicker = nrs.cards[i].0 as i32;
                    break;
                }
                return TWO_PAIR_START + (two_kind_h-1) * 156 + two_kind_l * 13 + kicker;
            } else {
                // Pair
                let mut kicker_0: i32 = -1;
                let mut kicker_1: i32 = -1;
                let mut kicker_2: i32 = -1;
                let mut i = 0;
                while i < 5 {
                    if nrs.cards[i].0 == two_kind_h as u8 {
                        i += 2; // Go faster
                        continue;
                    }
                    if kicker_0 == -1 {
                        kicker_0 = nrs.cards[i].0 as i32;
                    } else if kicker_1 == -1 {
                        kicker_1 = nrs.cards[i].0 as i32;
                    } else {
                        kicker_2 = nrs.cards[i].0 as i32;
                        break;
                    }
                    i += 1;
                }
                return PAIR_START + 1716 * two_kind_h + 132 * kicker_0 + 11 * kicker_1 + kicker_2;
            }
        } else {
            // High card
            return HIGH_CARD_START + 
                (nrs.cards[0].0 as i32) * 11880 +
                (nrs.cards[1].0 as i32) * 990 +
                (nrs.cards[2].0 as i32) * 90 +
                (nrs.cards[3].0 as i32) * 9 +
                (nrs.cards[4].0 as i32);
        }
    }
}