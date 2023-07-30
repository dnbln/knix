use crate::board_position::BoardIndex;
use crate::castle_rights::CastleRights;
use crate::cell_buffer::{RankCellBuffer, WholeBoardCellBuffer};
use crate::en_passant_target::EnPassantTarget;
use crate::piece::{BoardPiece, BoardPieceKind, PieceColor};
use crate::piece_move::{Move, MoveInfo};
use std::fmt;
use std::fmt::Formatter;

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct Board {
    repr: WholeBoardCellBuffer,
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.to_fen(f)
    }
}

impl Board {
    pub fn get_piece_at(&self, index: BoardIndex) -> Option<BoardPiece> {
        self.repr.get_piece(index)
    }

    fn parse_rank(fen: &str) -> Result<RankCellBuffer, ParseBoardError> {
        let mut cells = RankCellBuffer::init_empty();
        let mut idx = 0;
        for c in fen.chars() {
            match c {
                '1'..='8' => {
                    idx += c as u8 - b'0';
                }
                c => {
                    let piece = BoardPiece::try_from_fen_char(c)
                        .ok_or(ParseBoardError::InvalidFENPieceChar(c))?;

                    cells.set_piece(unsafe { BoardIndex::new_unchecked(idx) }, Some(piece));
                    idx += 1;
                }
            }
        }

        if idx < 8 {
            return Err(ParseBoardError::MissingFilesInFEN(idx, fen.to_string()));
        }

        Ok(cells)
    }

    pub fn empty_board() -> Self {
        Self {
            repr: WholeBoardCellBuffer::init_empty(),
        }
    }

    pub fn parse_from_fen(fen: &str) -> Result<Self, ParseBoardError> {
        let mut init = Self::empty_board();

        let mut iter = fen.split('/');

        for rank in (0..8).rev() {
            let rank_desc = iter.next().ok_or(ParseBoardError::MissingRank(rank))?;
            let parsed_rank = Self::parse_rank(rank_desc)?;
            init.repr.copy_rank_from(rank, &parsed_rank);
        }

        Ok(init)
    }

    pub fn to_fen(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for rank in (0..8).rev() {
            let mut empty_files = 0;

            let slice = self.repr.get_rank(rank);

            for (_, cell) in slice.iter_pieces() {
                match cell {
                    Some(p) => {
                        if empty_files != 0 {
                            write!(f, "{empty_files}")?;
                            empty_files = 0;
                        }

                        write!(f, "{}", p.to_fen_char())?;
                    }
                    None => {
                        empty_files += 1;
                    }
                }
            }

            if empty_files != 0 {
                write!(f, "{empty_files}")?;
            }

            if rank != 0 {
                write!(f, "/")?;
            }
        }

        Ok(())
    }

    pub fn piece_iterator<'a>(&'a self) -> impl Iterator<Item = (BoardIndex, BoardPiece)> + 'a {
        self.repr
            .iter_pieces()
            .filter_map(|(idx, p)| p.map(|p| (idx, p)))
    }

    pub fn all_possible_moves_for_turn<'a>(
        &'a self,
        turn: PieceColor,
        en_passant_target: Option<EnPassantTarget>,
        castle_rights: CastleRights,
    ) -> impl Iterator<Item = Move> + 'a {
        self.piece_iterator()
            .filter(move |(_, p)| p.color() == turn)
            .flat_map(move |(idx, p)| {
                p.moves_on_board(idx, self, en_passant_target, castle_rights)
                    .into_iter()
            })
    }

    pub fn board_after_move(&self, m: Move) -> (Self, MoveInfo) {
        let mut new_board = Self { repr: self.repr };

        let simple_move = |new_board: &mut Board, start: BoardIndex, end: BoardIndex| {
            let piece = new_board.repr.get_piece(start).unwrap();
            let captured = new_board.repr.get_piece(end);
            new_board.repr.set_piece(end, Some(piece));
            new_board.repr.set_piece(start, None);
            MoveInfo {
                captured,
                moved_piece_color: piece.color(),
                pawn_advanced: piece.kind() == BoardPieceKind::Pawn,
                revoked_castle_rights: match piece {
                    BoardPiece::WhiteRook => match start.get_pos() {
                        0 => CastleRights::WHITE_QUEEN_SIDE,
                        7 => CastleRights::WHITE_KING_SIDE,
                        _ => CastleRights::EMPTY,
                    },
                    BoardPiece::BlackRook => match start.get_pos() {
                        56 => CastleRights::BLACK_QUEEN_SIDE,
                        63 => CastleRights::BLACK_KING_SIDE,
                        _ => CastleRights::EMPTY,
                    },
                    BoardPiece::WhiteKing => {
                        CastleRights::WHITE_KING_SIDE | CastleRights::WHITE_QUEEN_SIDE
                    }
                    BoardPiece::BlackKing => {
                        CastleRights::BLACK_KING_SIDE | CastleRights::BLACK_QUEEN_SIDE
                    }
                    _ => CastleRights::EMPTY,
                },
                new_en_passant_target: match piece {
                    BoardPiece::WhitePawn if end.get_pos() - start.get_pos() == 16 => {
                        Some(EnPassantTarget(unsafe {
                            BoardIndex::new_unchecked(start.get_pos() + 8)
                        }))
                    }
                    BoardPiece::BlackPawn if start.get_pos() - end.get_pos() == 16 => {
                        Some(EnPassantTarget(unsafe {
                            BoardIndex::new_unchecked(start.get_pos() - 8)
                        }))
                    }
                    _ => None,
                },
            }
        };

        let move_info = match m {
            Move::Simple(start, end) => simple_move(&mut new_board, start, end),
            Move::EnPassant {
                en_passant_target,
                pawn_doing_en_passant,
                pawn_being_captured,
            } => {
                let pawn = new_board.repr.get_piece(pawn_doing_en_passant).unwrap();
                let captured_pawn = new_board.repr.get_piece(pawn_being_captured).unwrap();
                new_board.repr.set_piece(pawn_doing_en_passant, None);
                new_board.repr.set_piece(pawn_being_captured, None);
                new_board.repr.set_piece(en_passant_target.0, Some(pawn));
                MoveInfo {
                    moved_piece_color: pawn.color(),
                    revoked_castle_rights: CastleRights::EMPTY,
                    captured: Some(captured_pawn),
                    pawn_advanced: true,
                    new_en_passant_target: None,
                }
            }
            Move::Castle {
                king_from,
                king_to,
                rook_from,
                rook_to,
            } => {
                let mi = simple_move(&mut new_board, king_from, king_to);
                let mi2 = simple_move(&mut new_board, rook_from, rook_to);

                mi.combine_composite(mi2)
            }
        };

        (new_board, move_info)
    }

    pub fn check_move_validity(
        &self,
        turn: PieceColor,
        m: Move,
        en_passant_target: Option<EnPassantTarget>,
        castle_rights: CastleRights,
    ) -> bool {
        let (new_board, _mi) = self.board_after_move(m);
        let other_turn = turn.other();
        for m in new_board.all_possible_moves_for_turn(other_turn, en_passant_target, castle_rights)
        {
            if let Move::Simple(_start, end) = m {
                if new_board.get_piece_at(end) == Some(turn.king_of_color()) {
                    return false;
                }
            }
        }

        true
    }

    pub fn all_legal_moves_for_turn<'a>(
        &'a self,
        turn: PieceColor,
        en_passant_target: Option<EnPassantTarget>,
        castle_rights: CastleRights,
    ) -> impl Iterator<Item = Move> + 'a {
        self.all_possible_moves_for_turn(turn, en_passant_target, castle_rights)
            .filter(move |m| self.check_move_validity(turn, *m, en_passant_target, castle_rights))
    }

    pub fn to_visual(&self) -> BoardVisual {
        let mut buf = [0; 64];
        for (i, cell) in self.repr.iter_pieces() {
            buf[i.get_pos() as usize] = match cell {
                Some(p) => p.to_fen_char() as u8,
                None => 0,
            };
        }
        BoardVisual(buf)
    }
}

pub struct BoardVisual([u8; 64]);

impl fmt::Debug for BoardVisual {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", unsafe { std::str::from_utf8_unchecked(&self.0) })
    }
}

impl fmt::Display for BoardVisual {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "    ABCDEFGH\n   +--------+\n")?;
        for rank in (0..=7).rev() {
            write!(f, "{}  |", rank + 1)?;
            for file in 0..=7 {
                let ch = self.0[rank * 8 + file];
                if ch == 0 {
                    write!(f, " ")?;
                } else {
                    write!(f, "{}", ch as char)?;
                }
            }
            writeln!(f, "|  {}", rank + 1)?;
        }
        writeln!(f, "   +--------+\n    ABCDEFGH")
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ParseBoardError {
    #[error("rank {0} is missing from the FEN string")]
    MissingRank(u8),
    #[error("there were some missing files in the FEN string (got only {0} in {1:?})")]
    MissingFilesInFEN(u8, String),
    #[error("char {0} is an invalid FEN piece char")]
    InvalidFENPieceChar(char),
}
