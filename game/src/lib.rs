use std::collections::BTreeSet;

pub const DEFAULT_WORD: &str = "rust";
pub const MAX_ATTEMPTS: u32 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuessResult {
    Invalid,
    AlreadyGuessed,
    Hit,
    Miss,
    Won,
    Lost,
}

#[derive(Debug)]
pub struct Hangman {
    secret: String,
    mask: Vec<char>,
    guessed: BTreeSet<char>,
    attempts_left: u32,
}

impl Hangman {
    pub fn new(word: &str, max_attempts: u32) -> Self {
        let secret: String = word.chars().flat_map(char::to_lowercase).collect();
        let mask = vec!['_'; secret.chars().count()];

        Self {
            secret,
            mask,
            guessed: BTreeSet::new(),
            attempts_left: max_attempts,
        }
    }

    pub fn guess(&mut self, input: &str) -> GuessResult {
        if self.is_won() || self.is_lost() {
            return if self.is_won() {
                GuessResult::Won
            } else {
                GuessResult::Lost
            };
        }

        let Some(letter) = parse_letter(input) else {
            return GuessResult::Invalid;
        };

        if !self.guessed.insert(letter) {
            return GuessResult::AlreadyGuessed;
        }

        if self.secret.contains(letter) {
            for (index, secret_char) in self.secret.chars().enumerate() {
                if secret_char == letter {
                    self.mask[index] = letter;
                }
            }

            if self.is_won() {
                GuessResult::Won
            } else {
                GuessResult::Hit
            }
        } else {
            self.attempts_left = self.attempts_left.saturating_sub(1);

            if self.is_lost() {
                GuessResult::Lost
            } else {
                GuessResult::Miss
            }
        }
    }

    pub fn masked_word(&self) -> String {
        self.mask
            .iter()
            .map(|ch| ch.to_string())
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub fn guessed_letters(&self) -> String {
        self.guessed
            .iter()
            .map(|ch| ch.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub fn attempts_left(&self) -> u32 {
        self.attempts_left
    }

    pub fn is_won(&self) -> bool {
        !self.mask.is_empty() && self.mask.iter().all(|&ch| ch != '_')
    }

    pub fn is_lost(&self) -> bool {
        self.attempts_left == 0 && !self.is_won()
    }

    pub fn secret_word(&self) -> &str {
        &self.secret
    }
}

fn parse_letter(input: &str) -> Option<char> {
    let trimmed = input.trim();
    let mut chars = trimmed.chars();
    let letter = chars.next()?;

    if chars.next().is_some() {
        return None;
    }

    if !letter.is_alphabetic() {
        return None;
    }

    letter.to_lowercase().next()
}
