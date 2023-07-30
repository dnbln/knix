use std::fmt;
use std::fmt::Formatter;
use crate::board_position::{BoardIndex, BoardIndexDelta};
use crate::castle_rights::CastleRights;
use crate::en_passant_target::EnPassantTarget;
use crate::piece::{BoardPiece, PieceColor};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum Move {
    Simple(BoardIndex, BoardIndex),
    EnPassant {
        pawn_doing_en_passant: BoardIndex,
        pawn_being_captured: BoardIndex,
        en_passant_target: EnPassantTarget,
    },
    Castle {
        rook_from: BoardIndex,
        rook_to: BoardIndex,
        king_from: BoardIndex,
        king_to: BoardIndex,
    },
}


#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct MoveInfo {
    pub(crate) moved_piece_color: PieceColor,
    pub(crate) revoked_castle_rights: CastleRights,
    pub(crate) captured: Option<BoardPiece>,
    pub(crate) pawn_advanced: bool,
    pub(crate) new_en_passant_target: Option<EnPassantTarget>,
}

impl MoveInfo {
    pub fn combine_composite(self, mi2: MoveInfo) -> MoveInfo {
        debug_assert_eq!(self.moved_piece_color, mi2.moved_piece_color);
        MoveInfo {
            moved_piece_color: self.moved_piece_color,
            captured: self.captured.or(mi2.captured),
            revoked_castle_rights: self.revoked_castle_rights | mi2.revoked_castle_rights,
            pawn_advanced: self.pawn_advanced || mi2.pawn_advanced,
            new_en_passant_target: self.new_en_passant_target.or(mi2.new_en_passant_target),
        }
    }
}

impl Move {
    pub fn from_delta(pos: BoardIndex, delta: BoardIndexDelta) -> Option<Self> {
        Some(Self::Simple(pos, pos.checked_add(delta)?))
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Simple(start, end) => {
                write!(f, "{start:?} -> {end:?}")
            }
            Self::Castle {
                king_from,
                king_to,
                rook_from,
                rook_to,
            } => {
                write!(
                    f,
                    "C(K:{king_from:?} -> {king_to:?}, R:{rook_from:?} -> {rook_to:?})"
                )
            }
            Self::EnPassant {
                en_passant_target,
                pawn_doing_en_passant,
                pawn_being_captured,
            } => {
                write!(
                    f,
                    "EP({pawn_doing_en_passant:?} -> {:?}, capturing {pawn_being_captured:?})",
                    en_passant_target.0
                )
            }
        }
    }
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Simple(start, end) => {
                write!(f, "{start} -> {end}")
            }
            Self::Castle {
                king_from, king_to, ..
            } => {
                write!(f, "{king_from} -> {king_to}")
            }
            Self::EnPassant {
                en_passant_target,
                pawn_doing_en_passant,
                ..
            } => {
                write!(f, "{pawn_doing_en_passant:?} -> {:?}", en_passant_target.0)
            }
        }
    }
}
