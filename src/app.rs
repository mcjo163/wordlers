use std::io::{self, Write};
use termion::event::Key;

use crate::{util, ColorScheme, Game};

pub struct App<W: Write> {
    screen: W,
    game: Game,
    color_scheme: ColorScheme,
}

impl<W: Write> App<W> {
    pub fn new(screen: W) -> io::Result<Self> {
        let game = Game::new();
        let mut app = Self {
            screen,
            game,
            color_scheme: ColorScheme::from(catppuccin::PALETTE.mocha),
        };

        // Hide cursor on init.
        write!(app.screen, "{}", termion::cursor::Hide)?;
        app.repaint()?;
        Ok(app)
    }

    fn restart(&mut self) {
        self.game = Game::new();
    }

    pub fn handle_key(&mut self, key: Key) -> io::Result<()> {
        // After game is over, accept ENTER to restart.
        if self.game.has_won().is_some() {
            return match key {
                Key::Char('\n') => {
                    self.restart();
                    self.repaint()
                }
                _ => Ok(()),
            };
        }

        if match key {
            Key::Char('\n') => self.game.try_submit_guess(),
            Key::Char(c) => self.game.try_accept_letter(c),
            Key::Backspace => self.game.try_delete_letter(),
            _ => false,
        } {
            if let Some(won) = self.game.has_won() {
                if won {
                    self.game.set_message("You win!\nESC: quit, ENTER: new");
                } else {
                    self.game.set_message(&format!(
                        "The word was '{}'.\nESC: quit, ENTER: new",
                        self.game.answer()
                    ));
                }
            }
            self.repaint()
        } else {
            Ok(())
        }
    }

    pub fn repaint(&mut self) -> io::Result<()> {
        // Clear screen with appropriate background color.
        write!(
            self.screen,
            "{}{}",
            termion::color::Bg(self.color_scheme.game_bg),
            termion::clear::All,
        )?;

        self.draw_board()?;
        self.screen.flush()
    }

    fn draw_board(&mut self) -> io::Result<()> {
        let term_size = termion::terminal_size()?;

        if term_size.0 < Game::BOARD_SIZE.0 || term_size.1 < Game::BOARD_SIZE.1 {
            let resize_message = format!(
                "[{}×{}] is too small! Please make your terminal window bigger.",
                term_size.0, term_size.1
            );
            let wrapped_message = textwrap::wrap(&resize_message, term_size.0 as usize);
            let message_size = (
                wrapped_message
                    .iter()
                    .map(|line| line.len())
                    .max()
                    .unwrap_or(0) as u16,
                wrapped_message.len() as u16,
            );

            let (x, y) = util::get_centered_top_left(term_size, message_size);

            for (i, line) in wrapped_message.into_iter().enumerate() {
                let y_offset = i as u16;
                write!(
                    self.screen,
                    "{}{}{}{}",
                    termion::cursor::Goto(x, y + y_offset),
                    termion::color::Bg(self.color_scheme.game_bg),
                    termion::color::Fg(self.color_scheme.text_base),
                    line,
                )?;
            }
            Ok(())
        } else {
            let centered_top_left = util::get_centered_top_left(term_size, Game::BOARD_SIZE);
            self.game
                .paint(&mut self.screen, centered_top_left, &self.color_scheme)
        }
    }
}

impl<W: Write> Drop for App<W> {
    fn drop(&mut self) {
        // Reshow cursor on drop.
        write!(self.screen, "{}", termion::cursor::Show).unwrap();
    }
}
