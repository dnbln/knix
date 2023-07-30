use crate::board_position::{BoardColumn, BoardPosition};
use crate::game_state::GameState;

#[test]
fn can_parse_all_board_positions() {
    for col in [
        BoardColumn::A,
        BoardColumn::B,
        BoardColumn::C,
        BoardColumn::D,
        BoardColumn::E,
        BoardColumn::F,
        BoardColumn::G,
        BoardColumn::H,
    ] {
        for row in 1..8 {
            let str = format!("{col}{row}");
            assert_eq!(
                Ok(unsafe { BoardPosition::new_unchecked(row, col) }),
                str.parse()
            );
        }
    }
}

#[test]
fn correct_starting() {
    let _starting = GameState::starting();
}
