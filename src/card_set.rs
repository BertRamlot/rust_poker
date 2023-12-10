use std::fmt::{self};
use std::slice::{Iter, IterMut};
use crate::card::Card;


#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CardSet {
    pub cards: [Card; 7],
    size: usize
}

impl fmt::Display for CardSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cards_as_strings: Vec<String> = self.iter().map(|card| format!("{}", card)).collect();
        write!(f, "{}", cards_as_strings.join(" "))
    }
}

impl From<Vec<Card>> for CardSet {
    fn from(cards: Vec<Card>) -> Self {
        CardSet::new(&cards)
    }
}

impl From<Vec<u8>> for CardSet {
    fn from(c: Vec<u8>) -> Self {
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

impl From<&[u8]> for CardSet {
    fn from(c: &[u8]) -> Self {
        let vec = c.iter().map(|&x| x.into()).collect::<Vec<Card>>();
        vec.into()
    }
}

impl From<&str> for CardSet {
	fn from(s: &str) -> Self {
        // Format example:
        // "4h*2c*3c*As*9s*Qs" where '*' is any character (wildcard)
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
        if cards.len() > 7 {
            panic!("Too many cards provided. Must be less than 7 cards.")
        }
        let mut cs = CardSet {
            cards: [255.into(); 7],
            size: cards.len()
        };
        cs.set_cards_partial(cards, 0);
        cs
    }

    pub fn set_cards_partial(&mut self, cards: &[Card], offset: usize) {
        self.size = self.size.max(cards.len() + offset);
        for i in 0..cards.len() {
            self.cards[offset+i] = cards[i];
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn iter(&self) -> Iter<'_, Card> {
        self.cards[..self.size as usize].iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<'_, Card> {
        self.cards[..self.size as usize].iter_mut()
    }

    // Uses only 6*7 = 42 bits of the 64
    pub fn identifier(&self) -> u64 {
        let mut id = 0u64;
        for &card in self.iter() {
            id = (id << 6) | (card.0 as u64);
        }
        id
    }

    pub fn as_canonical(mut self) -> Self {
        self.canonicalize();
        self
    }

    // ~ 0.5 us
    pub fn canonicalize(&mut self) {
        const PRIMES: [u64; 13] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37, 41];
        
        let ranks: Vec<u8> = self.iter().map(|&c| c.rank()).collect();
        let suits: Vec<u8> = self.iter().map(|&c| c.suit()).collect();

        // Determine suit mapping
        let mut suit_count: [usize; 4] = [0; 4];
        for &suit in suits.iter() {
            suit_count[suit as usize] += 1;
        }
        let mut equal_suit_count_ranking: [u64; 4] = [1; 4];
        for (&rank, &suit) in ranks.iter().zip(suits.iter()) {
            equal_suit_count_ranking[suit as usize] *= PRIMES[rank as usize];
        }
        let mut inv_suit_mapping: Vec<usize> = (0..4).collect();
        inv_suit_mapping.sort_by_key(|&i| (suit_count[i], equal_suit_count_ranking[i]));

        let mut suit_mapping: [u8; 4] = [0; 4];
        for (i, &inv_suit_map) in inv_suit_mapping.iter().enumerate() {
            suit_mapping[inv_suit_map] = i as u8;
        }

        // Apply suit mapping and sort
        let mut out = ranks.iter().zip(suits.iter())
            .map(|(&rank, &suit)| Card(13 * suit_mapping[suit as usize] + rank))
            .collect::<Vec<Card>>();
        out.sort_by_key(|&x| 255u8 - x.0);

        self.set_cards_partial(&out, 0);
    }

    // ~ 0.25 us
    // Expects the card vector to be canonicalized
    // Expects the card vector to be of length 7
    pub fn evaluate(&self) -> i32 {
        // --- Outline algorithm ---
        // if Flush:
        //   if Straight:
        //     return STRAIGHT_FLUSH_START (includes ROYAL_FLUSH)
        //   else:
        //     return FLUSH
        //
        // if 4-of-kind:
        //   return FOUR_OF_KIND
        // if 3-of-kind && 2-of-kind:
        //   return FULL_HOUSE
        // 
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

        // TODO: Check if len 7
        // TODO: check if canonicalized

        const HIGH_CARD_START : i32      = 0 * (1 << 20) + 1;
        const PAIR_START : i32           = 1 * (1 << 20);
        const TWO_PAIR_START : i32       = 2 * (1 << 20);
        const THREE_OF_KIND_START : i32  = 3 * (1 << 20);
        const STRAIGHT_START : i32       = 4 * (1 << 20);
        const FLUSH_START : i32          = 5 * (1 << 20);
        const FULL_HOUSE_START : i32     = 6 * (1 << 20);
        const FOUR_OF_KIND_START : i32   = 7 * (1 << 20);
        const STRAIGHT_FLUSH_START : i32 = 8 * (1 << 20);

        let mut ranks: Vec<_> = self.iter().map(|&c| c.rank()).collect();

        let is_flush = self.cards[4].suit() == 3;
        if is_flush {
            let mut current_straight_count = 1;
            for i in 1..ranks.len() {
                if self.cards[i].suit() != 3 {
                    // Straight goes outside the flush suit
                    break;
                }
                if ranks[i-1] != ranks[i] + 1 {
                    current_straight_count = 1;
                    continue;
                }
                current_straight_count += 1;
                if current_straight_count == 4 && ranks[i] == 0 && ranks[0] == 12 {
                    // Low straight flush
                    return STRAIGHT_FLUSH_START;
                }
                if current_straight_count == 5 {
                    // Royal/Straight flush
                    return STRAIGHT_FLUSH_START + 1 + (ranks[i] as i32);
                }
            }
            // Flush (no straight found)
            return FLUSH_START + 
                (ranks[0] as i32) * 11880 + 
                (ranks[1] as i32) * 990 + 
                (ranks[2] as i32) * 90 + 
                (ranks[3] as i32) * 9 + 
                (ranks[4] as i32);
        }
        // CardSet is not a flush, so not 'Flush', 'Straight Flush', or 'Royal Flush'
        // Suit doesn't matter beyond this point => sort ranks as this is easier to work with
        ranks.sort_by(|a, b| b.cmp(&a));

        // n-of-a-kind
        let mut three_kind = 255u8;
        let mut two_kind_h = 255u8;
        let mut two_kind_l = 255u8;

        let mut kind_count: u8 = 1;
        for i in 1..ranks.len() {
            if ranks[i-1] == ranks[i] {
                kind_count += 1;
                if i != ranks.len()-1 {
                    // continue until we are at the end or the repeating pattern stops
                    continue;
                }
            }
            if kind_count == 4 {
                // Four of a kind
                let four_kind = ranks[i-1];
                let kicker_value: i32;
                if ranks[0] == four_kind {
                    // kicker is smaller than the rank of the four of a kind
                    kicker_value = ranks[4] as i32;
                } else {
                    // kicker is larger than the rank of the four of a kind
                    kicker_value = ranks[0] as i32;
                }
                return FOUR_OF_KIND_START + 13 * (four_kind as i32) + kicker_value;
            } else if three_kind == 255u8 && kind_count == 3 {
                three_kind = ranks[i-1];
            } else if two_kind_h == 255u8 && kind_count >= 2 {
                two_kind_h = ranks[i-1];
            } else if two_kind_l == 255u8 && kind_count == 2 {
                two_kind_l = ranks[i-1];
            }
            // reset for next repeating pattern
            kind_count = 1;
        }

        if three_kind != 255u8 && two_kind_h != 255u8 {
            // Full house
            return FULL_HOUSE_START + 13 * (three_kind as i32) + (two_kind_h as i32);
        }

        // Straight
        let mut current_straight_count = 1;
        for i in 1..ranks.len() {
            if ranks[i-1] != ranks[i] + 1 {
                if ranks[i-1] != ranks[i] {
                    current_straight_count = 1;
                }
                continue;
            }
            current_straight_count += 1;
            if current_straight_count == 4 && ranks[i] == 0 && ranks[0] == 12  {
                // Low straight
                return STRAIGHT_START;
            }
            if current_straight_count == 5 {
                // Normal straight
                return STRAIGHT_START + 1 + (ranks[i] as i32);
            }
        }
        
        if three_kind != 255u8 { // No need to check for "two_kind_h == -1", can't be a full-house
            // Three of a kind
            let mut kicker_0 = -1;
            let mut kicker_1 = -1;
            let mut i = 0;
            while i < 5 {
                if ranks[i] == three_kind {
                    i += 3; // Go faster
                    continue;
                }
                if kicker_0 == -1 {
                    kicker_0 = ranks[i] as i32;
                } else {
                    kicker_1 = ranks[i] as i32;
                    break;
                }
                i += 1;
            }
            return THREE_OF_KIND_START + 156 * (three_kind as i32) + 12 * kicker_0 + kicker_1;
        } else if two_kind_h != 255u8 {
            if two_kind_l != 255u8 {
                // Two pair
                let mut kicker: i32 = -1;
                let mut i = 0;
                while i < 5 {
                    if ranks[i] == two_kind_h || ranks[i] == two_kind_l {
                        i += 2; // Go faster
                        continue;
                    }
                    kicker = ranks[i] as i32;
                    break;
                }
                return TWO_PAIR_START + 156 * ((two_kind_h as i32) - 1) + 13 * (two_kind_l as i32) + kicker;
            } else {
                // Pair
                let mut kicker_0: i32 = -1;
                let mut kicker_1: i32 = -1;
                let mut kicker_2: i32 = -1;
                let mut i = 0;
                while i < 5 {
                    if ranks[i] == two_kind_h {
                        i += 2; // Go faster
                        continue;
                    }
                    if kicker_0 == -1 {
                        kicker_0 = ranks[i] as i32;
                    } else if kicker_1 == -1 {
                        kicker_1 = ranks[i] as i32;
                    } else {
                        kicker_2 = ranks[i] as i32;
                        break;
                    }
                    i += 1;
                }
                return PAIR_START + 1716 * (two_kind_h as i32) + 132 * kicker_0 + 11 * kicker_1 + kicker_2;
            }
        } else {
            // High card
            return HIGH_CARD_START + 
                (ranks[0] as i32) * 11880 +
                (ranks[1] as i32) * 990 +
                (ranks[2] as i32) * 90 +
                (ranks[3] as i32) * 9 +
                (ranks[4] as i32);
        }
    }
}