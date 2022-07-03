use std::fmt::{self, Write};


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Card(pub u8);


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