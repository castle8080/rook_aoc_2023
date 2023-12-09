use std::path::Path;
use std::collections::HashMap;

use crate::aocbase::{AOCError, AOCResult};
use crate::aocio::each_line;

use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref HAND_REGEX: Regex = Regex::new(r"^([AKQJT2-9]{5}) (\d+)").unwrap();
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, PartialOrd, Ord)]
pub enum Card {
    Joker = 0,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

impl Card {
    pub fn from_char(c: char) -> AOCResult<Card> {
        Ok(match c {
            '2' => Card::Two,
            '3' => Card::Three,
            '4' => Card::Four,
            '5' => Card::Five,
            '6' => Card::Six,
            '7' => Card::Seven,
            '8' => Card::Eight,
            '9' => Card::Nine,
            'T' => Card::Ten,
            'J' => Card::Jack,
            'Q' => Card::Queen,
            'K' => Card::King,
            'A' => Card::Ace,
            _ => { return Err(AOCError::ParseError(format!("Invalid card: {}", c))); }
        })
    }
}

#[derive(Debug, Eq, Hash, PartialEq, Copy, Clone, PartialOrd, Ord)]
pub enum HandType {
    HighCard = 0,
    OnePair,
    TwoPair,
    ThreeOfAKind,
    FullHouse,
    FourOfAKind,
    FiveOfAKind,
}

#[derive(Debug)]
pub struct Hand {
    cards: Vec<Card>,
    bid: i32,
    hand_type: HandType,
}

impl Hand {

    pub fn new(cards: Vec<Card>, bid: i32) -> Hand {
        let hand_type = Hand::get_hand_type(&cards);
        Hand {
            cards,
            bid,
            hand_type,
        }
    }

    pub fn rank_score(&self) -> i64 {
        // Calculates a single number for the rank from the hand type and inividual cards.
        // Think of each card being a digit. There are 14 cards so I can basically think
        // of a hand as being a single number in base 14.
        let digits = Card::Ace as i64 + 1;
        let mut score = self.hand_type as i64 * digits.pow(self.cards.len() as u32 + 1);

        for (idx, card) in self.cards.iter().enumerate() {
            let card_score = *card as i64;
            score += card_score * digits.pow(self.cards.len() as u32 - idx as u32 - 1);
        }

        score
    }

    fn get_hand_type(cards: &Vec<Card>) -> HandType {
        let mut count_counts = Hand::get_count_counts(cards.iter().filter(|card| **card != Card::Joker));
        let joker_count = cards.iter().filter(|card| **card == Card::Joker).count() as i32;

        // Adjust hand using jokers
        if joker_count > 0 {
            if let Some(max_count) = count_counts.keys().max() {
                let max_count = *max_count;
                let max_count_count = count_counts.get(&max_count).unwrap();

                count_counts.insert(max_count, max_count_count - 1);
                count_counts.insert(max_count + joker_count, 1);

            }
            else {
                // all jokers
                count_counts.insert(joker_count, 1);
            }
        }

        Hand::calculate_hand_type(&count_counts)
    }

    fn get_count_counts<'a>(cards: impl Iterator<Item = &'a Card>) -> HashMap<i32, i32> {
        let mut card_counts: HashMap<Card, i32> = HashMap::new();

        for c in cards {
            card_counts.insert(*c, card_counts.get(c).unwrap_or(&0) + 1);
        }

        let mut count_counts: HashMap<i32, i32> = HashMap::new();
        for count in card_counts.values() {
            count_counts.insert(*count, count_counts.get(count).unwrap_or(&0) + 1);
        }

        return count_counts;
    }

    fn calculate_hand_type(count_counts: &HashMap<i32, i32>) -> HandType {
        if count_counts.get(&5).unwrap_or(&0) == &1 {
            HandType::FiveOfAKind
        }
        else if count_counts.get(&4).unwrap_or(&0) == &1 {
            HandType::FourOfAKind
        }
        else if count_counts.get(&3).unwrap_or(&0) == &1 {
            if count_counts.get(&2).unwrap_or(&0) == &1 {
                HandType::FullHouse
            }
            else {
                HandType::ThreeOfAKind
            }
        }
        else if count_counts.get(&2).unwrap_or(&0) == &2 {
            HandType::TwoPair
        }
        else if count_counts.get(&2).unwrap_or(&0) == &1 {
            HandType::OnePair
        }
        else {
            HandType::HighCard
        }
    }

    pub fn parse(line: impl AsRef<str>, joker_type: Option<Card>) -> AOCResult<Hand> {
        let hand_cap = HAND_REGEX
            .captures(line.as_ref())
            .ok_or_else(|| AOCError::ParseError(format!("Invalid hand: {}", line.as_ref())))?;

        let mut cards = hand_cap
            .get(1)
            .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid group".into()))?
            .as_str()
            .chars()
            .map(Card::from_char)
            .collect::<AOCResult<Vec<Card>>>()?;

        if cards.len() != 5 {
            return Err(AOCError::InvalidRegexOperation(format!("Invalid card count: {}", cards.len())))
        }

        let bid = hand_cap
            .get(2)
            .ok_or_else(|| AOCError::InvalidRegexOperation("Invalid group".into()))?
            .as_str()
            .parse::<i32>()?;

        // Change normal card to joker?
        if let Some(joker_type) = joker_type {
            for card in cards.iter_mut() {
                if *card == joker_type {
                    *card = Card::Joker;
                }
            }
        }

        Ok(Hand::new(cards, bid))
    }
}

#[derive(Debug)]
pub struct Hands {
    pub hands: Vec<Hand>,
}

impl Hands {
    pub fn load(input: impl AsRef<Path>, joker_type: Option<Card>) -> AOCResult<Hands> {
        let mut hands: Vec<Hand> = Vec::new();
        each_line(input, |line| {
            hands.push(Hand::parse(line, joker_type)?);
            Ok(())
        })?;
        Ok(Hands { hands })
    }

    pub fn sort_hands(&mut self) {
        self.hands.sort_by_cached_key(|h| h.rank_score())
    }

    pub fn total_score(&self) -> i64 {
        self.hands.iter()
            .enumerate()
            .map(|(rank, hand)| (rank as i64 + 1) * hand.bid as i64)
            .sum()
    }
}

fn run_part(input: impl AsRef<Path>, joker_type: Option<Card>) -> AOCResult<String> {
    let mut hands = Hands::load(input, joker_type)?;
    hands.sort_hands();

    let result = hands.total_score();

    Ok(result.to_string())
}

pub fn part1(input: impl AsRef<Path>) -> AOCResult<String> {
    run_part(input, None)
}

pub fn part2(input: impl AsRef<Path>) -> AOCResult<String> {
    run_part(input, Some(Card::Jack))
}