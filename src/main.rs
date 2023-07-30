use rustyline::config::Configurer;
use rustyline::error::ReadlineError;
use rustyline::{Behavior, ColorMode};
use knix::game_state::GameState;

pub type R<T = ()> = anyhow::Result<T>;

fn do_read_fen(file: &str) -> R<GameState> {
    let fen = if file == "-" {
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf)?;
        buf
    } else {
        std::fs::read_to_string(file)?
    };

    Ok(GameState::parse_from_fen(&fen)?)
}

fn main() -> R {
    let mut editor = rustyline::Editor::<(), rustyline::history::DefaultHistory>::new().unwrap();
    editor.set_auto_add_history(true);
    editor.set_behavior(Behavior::PreferTerm);
    editor.set_color_mode(ColorMode::Enabled);

    let mut state = GameState::starting();
    println!("{}", std::mem::size_of::<GameState>());

    loop {
        match editor.readline(concat!("knix ", env!("CARGO_PKG_VERSION"), "> ")) {
            Ok(line) => {
                if line.starts_with("read-fen ") {
                    state = do_read_fen(line.strip_prefix("read-fen ").unwrap())?;
                }
            }
            Err(ReadlineError::Eof) => break,
            Err(e) => {
                eprintln!("{e}");
            }
        }
    }

    // let mut state = GameState::parse_from_fen(&fen).unwrap();
    dbg!(&state);

    let moves = state.legal_moves().collect::<Vec<_>>();
    dbg!(&moves);

    println!("{}", state.to_fen());

    state.perform_move(moves[5]);

    println!("{}", state.to_fen());

    println!("{}", state.board_to_visual());

    Ok(())
}
