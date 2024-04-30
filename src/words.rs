use rand::seq::SliceRandom;
use std::collections::HashSet;

/// Struct for holding dictionary data, choosing an answer,
/// and validating user guesses.
pub struct Words {
    answers: Vec<&'static str>,
    valid_guesses: HashSet<&'static str>,
}

impl Words {
    pub fn new() -> Self {
        // Wordle dictionaries sourced from
        // https://gist.github.com/scholtes/94f3c0303ba6a7768b47583aff36654d
        let la: Vec<_> = include_str!("../words/wordle-La.txt").lines().collect();
        let ta: Vec<_> = include_str!("../words/wordle-Ta.txt").lines().collect();

        let answers = la.clone();
        let valid_guesses = la.into_iter().chain(ta.into_iter()).collect();

        Self {
            answers,
            valid_guesses,
        }
    }

    /// Choose an answer from the possible answer dictionary.
    ///
    /// # Panics
    /// This method panics if the answers failed to load.
    pub fn get_answer(&self) -> &'static str {
        *self
            .answers
            .choose(&mut rand::thread_rng())
            .expect("Failed to load answers!")
    }

    /// Check if a word is a valid guess.
    pub fn valid_guess(&self, word: &str) -> bool {
        self.valid_guesses.contains(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn answers_load() {
        let words = Words::new();
        let answer = words.get_answer();
        assert_eq!(answer.len(), 5);
    }

    #[test]
    fn validates_guesses() {
        let words = Words::new();
        assert!(words.valid_guess("heart"));
        assert!(!words.valid_guess("abcde"));
    }
}
