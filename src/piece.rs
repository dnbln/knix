use crate::board::Board;
use crate::board_position::{BoardIndex, BoardIndexDelta};
use crate::castle_rights::CastleRights;
use crate::en_passant_target::EnPassantTarget;
use crate::piece_move::Move;

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum BoardPieceKind {
    Pawn = 1,
    Rook = 2,
    Knight = 3,
    Bishop = 4,
    Queen = 5,
    King = 6,
}

impl BoardPieceKind {
    pub fn of_color(self, color: PieceColor) -> BoardPiece {
        // Safety: only valid variants.
        unsafe { std::mem::transmute::<u8, BoardPiece>(self as u8 + 8 * (color as u8)) }
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum PieceColor {
    White = 0,
    Black = 1,
}

impl PieceColor {
    pub fn other(self) -> Self {
        match self {
            Self::Black => Self::White,
            Self::White => Self::Black,
        }
    }

    pub fn king_of_color(self) -> BoardPiece {
        match self {
            PieceColor::White => BoardPiece::WhiteKing,
            PieceColor::Black => BoardPiece::BlackKing,
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, Eq, PartialEq)]
#[repr(u8)]
pub enum BoardPiece {
    WhitePawn = 1,
    WhiteRook = 2,
    WhiteKnight = 3,
    WhiteBishop = 4,
    WhiteQueen = 5,
    WhiteKing = 6,

    BlackPawn = 9,
    BlackRook = 10,
    BlackKnight = 11,
    BlackBishop = 12,
    BlackQueen = 13,
    BlackKing = 14,
}

macro_rules! gen_fen {
    ($($char:literal => $name:ident),* $(,)?) => {
        impl BoardPiece {
            pub fn to_fen_char(self) -> char {
                match self {
                    $(Self::$name => $char,)*
                }
            }

            pub fn try_from_fen_char(c: char) -> Option<Self> {
                let p = match c {
                    $($char => Self::$name,)*
                    _ => return None,
                };

                Some(p)
            }
        }
    };
}

gen_fen!(
    'K' => WhiteKing,
    'k' => BlackKing,
    'Q' => WhiteQueen,
    'q' => BlackQueen,
    'B' => WhiteBishop,
    'b' => BlackBishop,
    'N' => WhiteKnight,
    'n' => BlackKnight,
    'R' => WhiteRook,
    'r' => BlackRook,
    'P' => WhitePawn,
    'p' => BlackPawn,
);

impl BoardPiece {
    pub(crate) unsafe fn from_repr(r: u8) -> Self {
        std::mem::transmute::<u8, Self>(r)
    }

    pub fn new(kind: BoardPieceKind, color: PieceColor) -> Self {
        kind.of_color(color)
    }

    #[track_caller]
    pub fn from_fen_char(c: char) -> Self {
        Self::try_from_fen_char(c).expect("Invalid FEN char")
    }

    pub fn kind(self) -> BoardPieceKind {
        self.split().0
    }

    pub fn color(self) -> PieceColor {
        self.split().1
    }

    pub fn split(self) -> (BoardPieceKind, PieceColor) {
        match self {
            BoardPiece::WhitePawn => (BoardPieceKind::Pawn, PieceColor::White),
            BoardPiece::BlackPawn => (BoardPieceKind::Pawn, PieceColor::Black),
            BoardPiece::WhiteRook => (BoardPieceKind::Rook, PieceColor::White),
            BoardPiece::BlackRook => (BoardPieceKind::Rook, PieceColor::Black),
            BoardPiece::WhiteKnight => (BoardPieceKind::Knight, PieceColor::White),
            BoardPiece::BlackKnight => (BoardPieceKind::Knight, PieceColor::Black),
            BoardPiece::WhiteBishop => (BoardPieceKind::Bishop, PieceColor::White),
            BoardPiece::BlackBishop => (BoardPieceKind::Bishop, PieceColor::Black),
            BoardPiece::WhiteQueen => (BoardPieceKind::Queen, PieceColor::White),
            BoardPiece::BlackQueen => (BoardPieceKind::Queen, PieceColor::Black),
            BoardPiece::WhiteKing => (BoardPieceKind::King, PieceColor::White),
            BoardPiece::BlackKing => (BoardPieceKind::King, PieceColor::Black),
        }
    }

    pub fn moves_on_board(
        &self,
        position: BoardIndex,
        b: &Board,
        en_passant_target: Option<EnPassantTarget>,
        _castle_rights: CastleRights,
    ) -> Vec<Move> {
        let self_color = self.color();

        let piece_at_delta = |d: BoardIndexDelta| {
            let new_pos = position.checked_add(d);
            match new_pos {
                Some(new_pos) => (true, b.get_piece_at(new_pos)),
                None => (false, None),
            }
        };

        let mut moves = Vec::new();

        let std_directional = |directions: &[(i8, i8)], moves: &mut Vec<Move>| {
            for dir in directions.iter().copied() {
                for i in 1..=7 {
                    let delta = BoardIndexDelta::new(dir.0 * i, dir.1 * i);
                    match piece_at_delta(delta) {
                        (true, None) => moves.push(Move::from_delta(position, delta).unwrap()),
                        (true, Some(p)) => {
                            if p.color() != self_color {
                                moves.push(Move::from_delta(position, delta).unwrap());
                            }
                            break;
                        }
                        (false, _) => break,
                    }
                }
            }
        };

        let pawn = |direction: i8, moves: &mut Vec<Move>| {
            if let (true, None) = piece_at_delta(BoardIndexDelta::delta_rank(direction)) {
                moves.push(
                    Move::from_delta(position, BoardIndexDelta::delta_rank(direction)).unwrap(),
                );

                // check for starting position.
                if direction == 1 && position.rank() == 2 || direction == -1 && position.rank() == 7
                {
                    if let (true, None) = piece_at_delta(BoardIndexDelta::delta_rank(2 * direction))
                    {
                        moves.push(
                            Move::from_delta(position, BoardIndexDelta::delta_rank(2 * direction))
                                .unwrap(),
                        );
                    }
                }
            }

            for delta_file in [-1, 1] {
                if let (true, Some(p)) = piece_at_delta(BoardIndexDelta::new(direction, delta_file))
                {
                    if p.color() != self.color() {
                        moves.push(
                            Move::from_delta(position, BoardIndexDelta::new(direction, delta_file))
                                .unwrap(),
                        );
                    }
                }
            }

            // en passant
            if let Some(ept) = en_passant_target {
                let delta = BoardIndexDelta::new(direction, 1);
                if ept.0 == position + delta {
                    moves.push(Move::EnPassant {
                        en_passant_target: ept,
                        pawn_being_captured: position + BoardIndexDelta::delta_file(1),
                        pawn_doing_en_passant: position,
                    })
                }

                let delta = BoardIndexDelta::new(direction, -1);
                if ept.0 == position + delta {
                    moves.push(Move::EnPassant {
                        en_passant_target: ept,
                        pawn_being_captured: position + BoardIndexDelta::delta_file(-1),
                        pawn_doing_en_passant: position,
                    })
                }
            }
        };

        let direct = |dirs: &[BoardIndexDelta], moves: &mut Vec<Move>| {
            for delta in dirs {
                match piece_at_delta(*delta) {
                    (true, None) => moves.push(Move::from_delta(position, *delta).unwrap()),
                    (true, Some(p)) if p.color() != self_color => {
                        moves.push(Move::from_delta(position, *delta).unwrap())
                    }
                    _ => {}
                }
            }
        };

        match self {
            BoardPiece::WhitePawn => pawn(1, &mut moves),
            BoardPiece::BlackPawn => pawn(-1, &mut moves),
            BoardPiece::WhiteRook | BoardPiece::BlackRook => {
                std_directional(&[(-1, 0), (1, 0), (0, -1), (0, 1)], &mut moves);
            }
            BoardPiece::WhiteKnight | BoardPiece::BlackKnight => direct(
                &[
                    BoardIndexDelta::new(-2, -1),
                    BoardIndexDelta::new(-2, 1),
                    BoardIndexDelta::new(-1, -2),
                    BoardIndexDelta::new(-1, 2),
                    BoardIndexDelta::new(1, -2),
                    BoardIndexDelta::new(1, 2),
                    BoardIndexDelta::new(2, -1),
                    BoardIndexDelta::new(2, 1),
                ],
                &mut moves,
            ),
            BoardPiece::WhiteBishop | BoardPiece::BlackBishop => {
                std_directional(&[(-1, -1), (-1, 1), (1, -1), (1, 1)], &mut moves)
            }
            BoardPiece::WhiteQueen | BoardPiece::BlackQueen => std_directional(
                &[
                    (-1, 0),
                    (1, 0),
                    (0, -1),
                    (0, 1),
                    (-1, -1),
                    (-1, 1),
                    (1, -1),
                    (1, 1),
                ],
                &mut moves,
            ),
            BoardPiece::WhiteKing | BoardPiece::BlackKing => direct(
                &[
                    BoardIndexDelta::new(-1, -1),
                    BoardIndexDelta::new(-1, 0),
                    BoardIndexDelta::new(-1, 1),
                    BoardIndexDelta::new(0, -1),
                    BoardIndexDelta::new(0, 1),
                    BoardIndexDelta::new(1, -1),
                    BoardIndexDelta::new(1, 0),
                    BoardIndexDelta::new(1, 1),
                ],
                &mut moves,
            ),
        }

        moves
    }
}
