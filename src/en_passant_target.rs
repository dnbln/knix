use crate::board_position::{BoardIndex, BoardPosition};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct EnPassantTarget(pub(crate) BoardIndex);

impl EnPassantTarget {
    pub fn from_fen(s: &str) -> Option<EnPassantTarget> {
        if s == "-" {
            return None;
        }

        s.parse::<BoardPosition>()
            .map(|it| EnPassantTarget(it.to_index()))
            .ok()
    }
}
