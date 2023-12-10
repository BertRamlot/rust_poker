use std::fmt;


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card(
    pub u8
);

const RANK_CHARS: &str = "23456789TJQKA";
const SUIT_CHARS: &str = "cdhs";

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rank = RANK_CHARS.chars().nth(self.rank() as usize);
        let suit = SUIT_CHARS.chars().nth(self.suit() as usize);
        
        match (rank, suit) {
            (Some(rank), Some(suit)) => write!(f, "{}{}", rank, suit),
            _ => Err(fmt::Error)
        }
    }
}

impl From<u8> for Card {
    fn from(c: u8) -> Self {
        Card(c)
    }
}

impl From<&str> for Card {
    fn from(s: &str) -> Self {
        if s.len() != 2 {
            panic!("Invalid input: String length must be 2 characters");
        }
        
        let rank = RANK_CHARS.find(s.chars().nth(0).unwrap());
        let suit = SUIT_CHARS.find(s.chars().nth(1).unwrap());

        match (rank, suit) {
            (Some(rank), Some(suit)) => Card::from((suit * 13 + rank) as u8),
            _ => panic!("Invalid input: Invalid card representation"),
        }
    }
}

impl Card {
    pub fn rank(&self) -> u8 {
        self.0 % 13
    }

    pub fn suit(&self) -> u8 {
        self.0 / 13
    }
}
