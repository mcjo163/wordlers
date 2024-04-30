use std::collections::HashMap;
use std::io::{self, Write};

use crate::{ColorScheme, Words};

/// Game state.
pub struct Game {
    rows: [BoardRow; 6],
    current_row: usize,
    answer: &'static str,
    words: Words,
    display_message: Option<String>,
    has_won: Option<bool>,
}

impl Game {
    /// The size (w, h) of the wordle board drawn with characters. Includes
    /// two extra rows at the bottom for a message.
    pub const BOARD_SIZE: (u16, u16) = (Cell::SIZE.0 * 5, Cell::SIZE.1 * 6 + 2);

    pub fn new() -> Self {
        let words = Words::new();
        let answer = words.get_answer();

        let mut game = Self {
            rows: [BoardRow::empty(); 6],
            current_row: 0,
            answer,
            words,
            display_message: None,
            has_won: None,
        };

        // Initialize game state.
        game.rows[0].current_cell = Some(0);
        game
    }

    fn get_current_row(&mut self) -> &mut BoardRow {
        &mut self.rows[self.current_row]
    }

    pub fn set_message(&mut self, message: &str) {
        self.display_message = Some(message.into());
    }

    pub fn clear_message(&mut self) {
        self.display_message = None;
    }

    pub fn has_won(&self) -> Option<bool> {
        self.has_won
    }

    pub fn answer(&self) -> &'static str {
        self.answer
    }

    /// Event handler for letter keys.
    /// Returning true indicates that the app should repaint.
    pub fn try_accept_letter(&mut self, letter: char) -> bool {
        self.clear_message();
        letter.is_ascii_alphabetic()
            && self
                .get_current_row()
                .try_accept_letter(letter.to_ascii_uppercase())
    }

    /// Event handler for backspace.
    /// Returning true indicates that the app should repaint.
    pub fn try_delete_letter(&mut self) -> bool {
        self.clear_message();
        self.get_current_row().try_delete_letter()
    }

    /// Event handler for the enter key.
    /// Returning true indicates that the app should repaint.
    pub fn try_submit_guess(&mut self) -> bool {
        self.clear_message();

        if let Some(guess) = self.get_current_row().get_final_word() {
            if !self.words.valid_guess(&guess) {
                self.set_message(&format!("'{guess}' is not a valid word!"));
                return true;
            }

            let answer = self.answer;
            self.get_current_row().check_guess(answer);

            if guess == self.answer {
                self.has_won = Some(true);
                return true;
            }

            if self.current_row < 5 {
                self.current_row += 1;
                self.get_current_row().current_cell = Some(0);
            } else {
                // Out of guesses!
                self.has_won = Some(false);
            }
            true
        } else {
            false
        }
    }

    pub fn paint(
        &self,
        screen: &mut impl Write,
        top_left: (u16, u16),
        colors: &ColorScheme,
    ) -> io::Result<()> {
        let (x, y) = top_left;

        for (i, row) in self.rows.iter().enumerate() {
            let y_offset = (i as u16) * Cell::SIZE.1;
            row.paint(screen, (x, y + y_offset), colors, i == self.current_row)?;
        }

        if let Some(message) = &self.display_message {
            // Write up to two wrapped message lines beneath the board.
            let lines = textwrap::wrap(message, Self::BOARD_SIZE.0 as usize);
            for i in 0..2 {
                if let Some(line) = lines.get(i) {
                    let y_offset = Self::BOARD_SIZE.1 - 2 + (i as u16);
                    write!(
                        screen,
                        "{}{}{}{}",
                        termion::cursor::Goto(x, y + y_offset),
                        termion::color::Bg(colors.game_bg),
                        termion::color::Fg(colors.text_base),
                        line,
                    )?;
                }
            }
        }

        Ok(())
    }
}

/// Single row of the game board.
#[derive(Copy, Clone)]
struct BoardRow {
    cells: [Cell; 5],
    current_cell: Option<usize>,
}

impl BoardRow {
    fn empty() -> Self {
        Self {
            cells: [Cell::Pending(None); 5],
            current_cell: None,
        }
    }

    /// If all cells are filled, return the string they make.
    /// Otherwise, return None.
    fn get_final_word(&self) -> Option<String> {
        let mut word = String::new();
        for cell in self.cells {
            let letter = match cell {
                Cell::Pending(None) => return None,
                Cell::Pending(Some(l))
                | Cell::NotInWord(l)
                | Cell::InWord(l)
                | Cell::Correct(l) => l,
            };
            word.push(letter);
        }
        Some(word.to_ascii_lowercase())
    }

    fn get_current_cell(&mut self) -> Option<&mut Cell> {
        self.current_cell.map(|i| &mut self.cells[i])
    }

    /// Row handler for letter keys.
    /// Returning true indicates that the app should repaint.
    fn try_accept_letter(&mut self, letter: char) -> bool {
        if let Some(i) = self.current_cell {
            if i < 5 {
                *self.get_current_cell().unwrap() = Cell::Pending(Some(letter));
                self.current_cell = Some(i + 1);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Row handler for backspace.
    /// Returning true indicates that the app should repaint.
    fn try_delete_letter(&mut self) -> bool {
        if let Some(i) = self.current_cell {
            if i > 0 {
                self.current_cell = Some(i - 1);
                *self.get_current_cell().unwrap() = Cell::Pending(None);
                true
            } else {
                false
            }
        } else {
            false
        }
    }

    /// Finalize the cells of this row according to the answer.
    ///
    /// This is the "meat" of the wordle logic.
    ///
    /// # Panics
    /// Panics if called on a row that is not complete.
    fn check_guess(&mut self, answer: &str) {
        let guess = self
            .get_final_word()
            .expect("Should only be called when all letters are here");

        // Closure to turn a string into a map of letters
        // to the indices at which they occur.
        let get_char_map = |s: &str| {
            s.char_indices().fold(HashMap::new(), |mut map, (i, c)| {
                let entry = map.entry(c).or_insert(Vec::new());
                entry.push(i);
                map
            })
        };

        let guess_chars = get_char_map(&guess);
        let answer_chars = get_char_map(answer);

        // Consider each letter individually.
        for (letter, guess_indices) in guess_chars.into_iter() {
            // If the letter is in the answer...
            if let Some(answer_indices) = answer_chars.get(&letter) {
                let mut num_reported = 0;
                let mut potential_yellows = Vec::with_capacity(guess_indices.len());

                // Start with exact matches.
                for i in guess_indices {
                    if answer_indices.contains(&i) {
                        self.cells[i].correct();
                        num_reported += 1;
                    } else {
                        potential_yellows.push(i);
                    }
                }

                // Fill in yellows from left to right as long as there are
                // "un-greened" instances of this letter in the answer.
                for i in potential_yellows {
                    if num_reported < answer_indices.len() {
                        self.cells[i].in_word();
                        num_reported += 1;
                    } else {
                        self.cells[i].not_in_word();
                    }
                }
            } else {
                // This letter is not in the answer.
                for i in guess_indices {
                    self.cells[i].not_in_word();
                }
            }
        }
    }

    fn paint(
        &self,
        screen: &mut impl Write,
        top_left: (u16, u16),
        colors: &ColorScheme,
        active: bool,
    ) -> io::Result<()> {
        let (x, y) = top_left;

        for (i, cell) in self.cells.iter().enumerate() {
            let x_offset = (i as u16) * Cell::SIZE.0;
            cell.paint(
                screen,
                (x + x_offset, y),
                colors,
                active,
                self.current_cell == Some(i),
            )?;
        }

        Ok(())
    }
}

/// A single letter cell.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum Cell {
    Pending(Option<char>),
    NotInWord(char),
    InWord(char),
    Correct(char),
}

impl Cell {
    /// Size (w, h) of a single cell, for reference.
    const SIZE: (u16, u16) = (5, 3);

    /// Update `Cell::Pending` variant to `Cell::NotInWord`.
    /// Has no effect if called on a finalized or empty cell.
    fn not_in_word(&mut self) {
        if let Self::Pending(Some(l)) = self {
            *self = Self::NotInWord(*l);
        }
    }

    /// Update `Cell::Pending` variant to `Cell::InWord`.
    /// Has no effect if called on a finalized or empty cell.
    fn in_word(&mut self) {
        if let Self::Pending(Some(l)) = self {
            *self = Self::InWord(*l);
        }
    }

    /// Update `Cell::Pending` variant to `Cell::Correct`.
    /// Has no effect if called on a finalized or empty cell.
    fn correct(&mut self) {
        if let Self::Pending(Some(l)) = self {
            *self = Self::Correct(*l);
        }
    }

    fn paint(
        &self,
        screen: &mut impl Write,
        top_left: (u16, u16),
        colors: &ColorScheme,
        row_active: bool,
        cell_active: bool,
    ) -> io::Result<()> {
        let (x, y) = top_left;
        let bg_color = colors.game_bg;
        let cell_char = self.get_char();

        let base_color = if row_active {
            if cell_active {
                colors.cell_active
            } else {
                colors.cell_row_active
            }
        } else {
            colors.cell_base
        };

        let (text_color, cell_color) = match *self {
            Self::Pending(_) | Self::NotInWord(_) => (colors.text_base, base_color),
            Self::InWord(_) => (colors.text_inverted, colors.cell_in_word),
            Self::Correct(_) => (colors.text_inverted, colors.cell_correct),
        };

        write!(
            screen,
            "{}{}{} ▄▄▄ {} █{}{}{}{}{}█ {} ▀▀▀ ",
            termion::cursor::Goto(x, y), // Row 1.
            termion::color::Bg(bg_color),
            termion::color::Fg(cell_color),
            termion::cursor::Goto(x, y + 1), // Row 2.
            termion::color::Bg(cell_color),
            termion::color::Fg(text_color),
            cell_char,
            termion::color::Bg(bg_color),
            termion::color::Fg(cell_color),
            termion::cursor::Goto(x, y + 2), // Row 3.
        )
    }

    /// Get the character to display.
    fn get_char(&self) -> char {
        match *self {
            Self::Pending(None) => ' ',
            Self::Pending(Some(c)) => c,
            Self::NotInWord(c) | Self::InWord(c) | Self::Correct(c) => c,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_pending_row_for_str(s: &str) -> BoardRow {
        assert_eq!(s.len(), 5);
        let cells: Vec<_> = s
            .chars()
            .map(|c| Cell::Pending(Some(c.to_ascii_uppercase())))
            .collect();
        BoardRow {
            cells: [cells[0], cells[1], cells[2], cells[3], cells[4]],
            current_cell: None,
        }
    }

    #[test]
    fn checks_guesses_correctly() {
        let answer = "heart";

        let mut row = get_pending_row_for_str("heart");
        row.check_guess(answer);
        assert_eq!(
            row.cells,
            [
                Cell::Correct('H'),
                Cell::Correct('E'),
                Cell::Correct('A'),
                Cell::Correct('R'),
                Cell::Correct('T'),
            ]
        );

        let mut row = get_pending_row_for_str("sound");
        row.check_guess(answer);
        assert_eq!(
            row.cells,
            [
                Cell::NotInWord('S'),
                Cell::NotInWord('O'),
                Cell::NotInWord('U'),
                Cell::NotInWord('N'),
                Cell::NotInWord('D'),
            ]
        );

        let mut row = get_pending_row_for_str("earth");
        row.check_guess(answer);
        assert_eq!(
            row.cells,
            [
                Cell::InWord('E'),
                Cell::InWord('A'),
                Cell::InWord('R'),
                Cell::InWord('T'),
                Cell::InWord('H'),
            ]
        );
    }

    #[test]
    fn handles_multi_letters_correctly() {
        let mut row = get_pending_row_for_str("gucci");
        row.check_guess("cacti");
        assert_eq!(
            row.cells,
            [
                Cell::NotInWord('G'),
                Cell::NotInWord('U'),
                Cell::Correct('C'),
                Cell::InWord('C'),
                Cell::Correct('I'),
            ]
        );

        let mut row = get_pending_row_for_str("bocce");
        row.check_guess("coast");
        assert_eq!(
            row.cells,
            [
                Cell::NotInWord('B'),
                Cell::Correct('O'),
                Cell::InWord('C'),
                Cell::NotInWord('C'),
                Cell::NotInWord('E'),
            ]
        );
    }
}
