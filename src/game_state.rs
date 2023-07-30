use crate::board::{Board, BoardVisual, ParseBoardError};
use crate::castle_rights::{CastleRights, InvalidCastleRight};
use crate::clocks::{FullMoveCounter, HalfMoveClock};
use crate::en_passant_target::EnPassantTarget;
use crate::piece::PieceColor;
use std::fmt;
use std::fmt::Formatter;
use std::num::ParseIntError;
use crate::piece_move::Move;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct GameState {
    board: Board,
    next_move: PieceColor,
    castling_rights: CastleRights,
    en_passant_target: Option<EnPassantTarget>,
    half_move_clock: HalfMoveClock,
    full_move_counter: FullMoveCounter,
}

impl GameState {
    pub fn starting() -> Self {
        Self::parse_from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
    }

    // pub fn list_valid_moves(&self, side: PieceColor) -> impl Iterator<Item = Move> {}

    pub fn parse_from_fen(fen: &str) -> Result<Self, ParseGameStateError> {
        let mut iter = fen.split(' ');

        let mut next = |field_count: usize| {
            iter.next()
                .ok_or(ParseGameStateError::MissingFields { field_count })
        };

        let board = next(0)?;
        let next_move = next(1)?;
        let rights = next(2)?;
        let en_passant_target = next(3)?;
        let half_moves = next(4)?;
        let full_moves = next(5)?;

        let board = Board::parse_from_fen(board)?;

        let next_move = match next_move {
            "w" => PieceColor::White,
            "b" => PieceColor::Black,
            _ => return Err(ParseGameStateError::InvalidNextMove(next_move.to_string())),
        };

        let castling_rights = CastleRights::rights_from_fen_str(rights)?;
        let en_passant_target = EnPassantTarget::from_fen(en_passant_target);
        let half_move_clock = HalfMoveClock::new_from_clock(half_moves.parse()?);
        let full_move_counter = FullMoveCounter::new_from_counter(full_moves.parse()?);

        Ok(Self {
            board,
            next_move,
            castling_rights,
            en_passant_target,
            half_move_clock,
            full_move_counter,
        })
    }

    pub fn board_to_fen(&self) -> String {
        struct BoardFenWrapper<'a>(&'a Board);
        impl<'a> fmt::Display for BoardFenWrapper<'a> {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                self.0.to_fen(f)
            }
        }
        format!("{}", BoardFenWrapper(&self.board))
    }

    pub fn to_fen(&self) -> String {
        use std::fmt::Write;

        let mut fen = self.board_to_fen();

        match self.next_move {
            PieceColor::White => fen.push_str(" w "),
            PieceColor::Black => fen.push_str(" b "),
        }

        let mut has_rights = false;
        let cr = self.castling_rights;
        if cr.has_rights(CastleRights::WHITE_KING_SIDE) {
            fen.push('K');
            has_rights = true;
        }
        if cr.has_rights(CastleRights::WHITE_QUEEN_SIDE) {
            fen.push('Q');
            has_rights = true;
        }
        if cr.has_rights(CastleRights::BLACK_KING_SIDE) {
            fen.push('k');
            has_rights = true;
        }
        if cr.has_rights(CastleRights::BLACK_QUEEN_SIDE) {
            fen.push('q');
            has_rights = true;
        }
        if !has_rights {
            fen.push('-');
        }
        fen.push(' ');

        match self.en_passant_target {
            Some(ept) => {
                write!(fen, "{}", ept.0).unwrap();
            }
            None => {
                fen.push('-');
            }
        }

        write!(
            fen,
            " {} {}",
            self.half_move_clock.get(), self.full_move_counter.get()
        )
        .unwrap();

        fen
    }

    pub fn legal_moves<'a>(&'a self) -> impl Iterator<Item = Move> + 'a {
        self.board.all_legal_moves_for_turn(
            self.next_move,
            self.en_passant_target,
            self.castling_rights,
        )
    }

    pub fn perform_move(&mut self, m: Move) {
        let (b, mi) = self.board.board_after_move(m);
        self.board = b;

        debug_assert_eq!(self.next_move, mi.moved_piece_color);

        if mi.pawn_advanced || mi.captured.is_some() {
            self.half_move_clock.reset();
        } else {
            self.half_move_clock.advance();
        }

        if mi.moved_piece_color == PieceColor::Black {
            self.full_move_counter.inc();
        }

        self.en_passant_target = mi.new_en_passant_target;
        self.castling_rights.revoke(mi.revoked_castle_rights);
        self.next_move = mi.moved_piece_color.other();
    }

    pub fn board_to_visual(&self) -> BoardVisual {
        self.board.to_visual()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseGameStateError {
    #[error("missing fields (got only {field_count})")]
    MissingFields { field_count: usize },
    #[error("parse int: {0}")]
    ParseInt(#[from] ParseIntError),
    #[error("invalid board: {0}")]
    InvalidBoard(#[from] ParseBoardError),
    #[error("invalid next move: {0}")]
    InvalidNextMove(String),
    #[error("invalid castle right: {0:?}")]
    InvalidCastleRight(#[from] InvalidCastleRight),
}
