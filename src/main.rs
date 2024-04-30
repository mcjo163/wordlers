use std::io;
use std::thread;

use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use termion::screen::IntoAlternateScreen;

use tokio::select;
use tokio::signal::unix::{signal, SignalKind};
use tokio::sync::mpsc;

mod app;
use app::App;

mod color_scheme;
pub use color_scheme::ColorScheme;

mod game;
pub use game::Game;

mod util;

mod words;
pub use words::Words;

/// Spawn a thread that sends termion key events asynchronously.
fn spawn_input_thread() -> mpsc::UnboundedReceiver<Key> {
    let (tx, rx) = mpsc::unbounded_channel();
    thread::spawn(move || {
        let stdin = io::stdin();
        for key in stdin.keys() {
            if let Ok(key) = key {
                if tx.send(key).is_err() {
                    break;
                }
            }
        }
    });
    rx
}

/// Run the game.
async fn run() -> io::Result<()> {
    // Set up resize and key event listeners.
    let mut resized_events = signal(SignalKind::window_change())?;
    let mut key_events = spawn_input_thread();

    // Open an "Alternate Screen" that will restore terminal session on drop.
    let screen = io::stdout().into_raw_mode()?.into_alternate_screen()?;
    let mut app = App::new(screen)?;

    loop {
        select! {
            Some(key) = key_events.recv() => {
                match key {
                    Key::Esc => break,
                    k => app.handle_key(k)?,
                }
            },
            _ = resized_events.recv() => {
                app.repaint()?;
            },
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            eprintln!("err: {e}");
            std::process::exit(1);
        }
    }
}
