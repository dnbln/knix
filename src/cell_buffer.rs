use crate::board_position::BoardIndex;
use crate::piece::BoardPiece;
use std::fmt;
use std::fmt::Formatter;
use std::ops::Range;

#[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
pub(crate) struct BoardCellRepr {
    r: u8, // 2 pieces in a single 'cell' (as in memory cell,
           // u8 or byte, not board cell).
           // the first 4 bits of the u8 are for one piece.
           // the last 4 bits of the u8 are for the second piece.
           // if no piece, then the 4 corresponding bits are all 0.
           // if there is a piece, then the bits are not all 0 and
           // taking their numerical value maps exactly to a variant
           // of the BoardPiece enum.
           // This allows us to represent a whole board with just 32 bytes.
}

impl fmt::Debug for BoardCellRepr {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self.get_piece0() {
            None => write!(f, "-")?,
            Some(p) => write!(f, "{:?}", p)?,
        }
        match self.get_piece1() {
            None => write!(f, "-"),
            Some(p) => write!(f, "{:?}", p),
        }
    }
}

impl BoardCellRepr {
    pub fn get_piece0(self) -> Option<BoardPiece> {
        match self.r & 0xF0 {
            0 => None,
            r => Some(unsafe { BoardPiece::from_repr(r >> 4) }),
        }
    }

    pub fn get_piece1(self) -> Option<BoardPiece> {
        match self.r & 0x0F {
            0 => None,
            r => Some(unsafe { BoardPiece::from_repr(r) }),
        }
    }

    pub fn set_piece0(&mut self, p0: Option<BoardPiece>) {
        self.r &= 0x0F;
        match p0 {
            Some(p0) => {
                self.r |= (p0 as u8) << 4;
            }
            None => {}
        }
    }

    pub fn set_piece1(&mut self, p1: Option<BoardPiece>) {
        self.r &= 0xF0;
        match p1 {
            Some(p1) => {
                self.r |= p1 as u8;
            }
            None => {}
        }
    }

    pub fn from_pieces(p0: Option<BoardPiece>, p1: Option<BoardPiece>) -> Self {
        Self {
            r: (p0.map_or(0, |it| it as u8) << 4 | p1.map_or(0, |it| it as u8)),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct BoardCellBuffer<const N: usize> {
    pub(crate) repr: [BoardCellRepr; N],
}

impl<const N: usize> BoardCellBuffer<N> {
    pub fn set_piece(&mut self, index: BoardIndex, p: Option<BoardPiece>) {
        let cell = index.get_pos() / 2;
        debug_assert!((cell as usize) < N);
        let pos_in_cell = index.get_pos() % 2;
        let f = match pos_in_cell {
            0 => BoardCellRepr::set_piece0,
            1 => BoardCellRepr::set_piece1,
            _ => unreachable!(),
        };
        f(&mut self.repr[cell as usize], p);
    }

    pub fn get_piece(&self, index: BoardIndex) -> Option<BoardPiece> {
        let cell = index.get_pos() / 2;
        debug_assert!((cell as usize) < N);
        let pos_in_cell = index.get_pos() % 2;
        let f = match pos_in_cell {
            0 => BoardCellRepr::get_piece0,
            1 => BoardCellRepr::get_piece1,
            _ => unreachable!(),
        };
        f(self.repr[cell as usize])
    }

    pub fn copy<const N2: usize>(&self, start: u8) -> BoardCellBuffer<N2> {
        let mut new_buffer = BoardCellBuffer {
            repr: [BoardCellRepr::from_pieces(None, None); N2],
        };

        new_buffer
            .repr
            .copy_from_slice(&self.repr[start as usize..start as usize + N2]);

        new_buffer
    }

    pub fn iter_pieces<'a>(
        &'a self,
    ) -> impl Iterator<Item = (BoardIndex, Option<BoardPiece>)> + 'a {
        self.repr.iter().enumerate().flat_map(|(idx, r)| {
            [
                (
                    unsafe { BoardIndex::new_unchecked((idx * 2) as u8) },
                    r.get_piece0(),
                ),
                (
                    unsafe { BoardIndex::new_unchecked((idx * 2 + 1) as u8) },
                    r.get_piece1(),
                ),
            ]
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub(crate) struct WholeBoardCellBuffer(BoardCellBuffer<32>);

impl WholeBoardCellBuffer {
    pub(crate) fn init_empty() -> Self {
        Self(BoardCellBuffer {
            repr: [BoardCellRepr { r: 0 }; 32],
        })
    }

    pub fn set_piece(&mut self, index: BoardIndex, p: Option<BoardPiece>) {
        self.0.set_piece(index, p)
    }

    pub fn get_piece(&self, index: BoardIndex) -> Option<BoardPiece> {
        self.0.get_piece(index)
    }

    pub fn get_rank(&self, rank: u8) -> RankCellBuffer {
        RankCellBuffer(self.0.copy(rank * 4))
    }

    pub fn iter_pieces<'a>(
        &'a self,
    ) -> impl Iterator<Item = (BoardIndex, Option<BoardPiece>)> + 'a {
        self.0.iter_pieces()
    }

    pub fn copy_rank_from(&mut self, rank: u8, rank_buffer: &RankCellBuffer) {
        self.0.repr[Self::index_range_for_rank(rank as usize)].copy_from_slice(&rank_buffer.0.repr);
    }

    fn index_range_for_rank(r: usize) -> Range<usize> {
        r * 4..(r + 1) * 4
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct RankCellBuffer(BoardCellBuffer<4>);

impl RankCellBuffer {
    pub(crate) fn init_empty() -> Self {
        Self(BoardCellBuffer {
            repr: [BoardCellRepr { r: 0 }; 4],
        })
    }
}

impl RankCellBuffer {
    pub fn set_piece(&mut self, index: BoardIndex, p: Option<BoardPiece>) {
        self.0.set_piece(index, p)
    }

    pub fn get_piece(&self, index: BoardIndex) -> Option<BoardPiece> {
        self.0.get_piece(index)
    }

    pub fn iter_pieces<'a>(
        &'a self,
    ) -> impl Iterator<Item = (BoardIndex, Option<BoardPiece>)> + 'a {
        self.0.iter_pieces()
    }
}

#[cfg(test)]
mod tests {
    use crate::board_position::BoardIndex;
    use crate::cell_buffer::{BoardCellBuffer, BoardCellRepr, WholeBoardCellBuffer};
    use crate::piece::BoardPiece;

    #[test]
    fn get_pieces() {
        let x = BoardCellRepr::from_pieces(None, None);
        assert_eq!(x.get_piece0(), None);
        assert_eq!(x.get_piece1(), None);

        let x = BoardCellRepr::from_pieces(None, Some(BoardPiece::WhitePawn));
        assert_eq!(x.get_piece0(), None);
        assert_eq!(x.get_piece1(), Some(BoardPiece::WhitePawn));

        let x = BoardCellRepr::from_pieces(Some(BoardPiece::BlackBishop), Some(BoardPiece::BlackPawn));
        assert_eq!(x.get_piece0(), Some(BoardPiece::BlackBishop));
        assert_eq!(x.get_piece1(), Some(BoardPiece::BlackPawn));

        let x = BoardCellRepr::from_pieces(Some(BoardPiece::BlackBishop), None);
        assert_eq!(x.get_piece0(), Some(BoardPiece::BlackBishop));
        assert_eq!(x.get_piece1(), None);
    }

    #[test]
    fn with_set_piece() {
        let mut x = BoardCellRepr::from_pieces(None, None);
        assert_eq!(x.get_piece0(), None);
        assert_eq!(x.get_piece1(), None);

        x.set_piece0(Some(BoardPiece::WhiteKing));
        assert_eq!(x.get_piece0(), Some(BoardPiece::WhiteKing));
        assert_eq!(x.get_piece1(), None);

        x.set_piece1(Some(BoardPiece::BlackPawn));
        assert_eq!(x.get_piece0(), Some(BoardPiece::WhiteKing));
        assert_eq!(x.get_piece1(), Some(BoardPiece::BlackPawn));

        x.set_piece0(None);
        assert_eq!(x.get_piece0(), None);
        assert_eq!(x.get_piece1(), Some(BoardPiece::BlackPawn));

        x.set_piece1(None);
        assert_eq!(x.get_piece0(), None);
        assert_eq!(x.get_piece1(), None);
    }

    #[test]
    fn get_and_set_pieces_in_whole_board_buffer() {
        fn count_pieces(b: &WholeBoardCellBuffer) -> usize {
            b.iter_pieces().filter(|(_, it)| it.is_some()).count()
        }

        let mut buffer = WholeBoardCellBuffer::init_empty();
        assert_eq!(0, count_pieces(&buffer));
        let bi0 = BoardIndex::new(0).unwrap();
        let bi1 = BoardIndex::new(1).unwrap();
        let bi2 = BoardIndex::new(2).unwrap();
        buffer.set_piece(bi0, Some(BoardPiece::BlackPawn));
        assert_eq!(1, count_pieces(&buffer));
        assert_eq!(Some(BoardPiece::BlackPawn), buffer.get_piece(bi0));
        assert_eq!(None, buffer.get_piece(bi1));
        assert_eq!(None, buffer.get_piece(bi2));
        buffer.set_piece(bi1, Some(BoardPiece::BlackQueen));
        assert_eq!(2, count_pieces(&buffer));
        assert_eq!(Some(BoardPiece::BlackPawn), buffer.get_piece(bi0));
        assert_eq!(Some(BoardPiece::BlackQueen), buffer.get_piece(bi1));
        assert_eq!(None, buffer.get_piece(bi2));
        buffer.set_piece(bi2, Some(BoardPiece::WhiteKing));
        assert_eq!(3, count_pieces(&buffer));
        assert_eq!(Some(BoardPiece::BlackPawn), buffer.get_piece(bi0));
        assert_eq!(Some(BoardPiece::BlackQueen), buffer.get_piece(bi1));
        assert_eq!(Some(BoardPiece::WhiteKing), buffer.get_piece(bi2));
        buffer.set_piece(bi2, Some(BoardPiece::BlackKing));
        assert_eq!(3, count_pieces(&buffer));
        assert_eq!(Some(BoardPiece::BlackPawn), buffer.get_piece(bi0));
        assert_eq!(Some(BoardPiece::BlackQueen), buffer.get_piece(bi1));
        assert_eq!(Some(BoardPiece::BlackKing), buffer.get_piece(bi2));
        buffer.set_piece(bi1, None);
        assert_eq!(2, count_pieces(&buffer));
        assert_eq!(Some(BoardPiece::BlackPawn), buffer.get_piece(bi0));
        assert_eq!(None, buffer.get_piece(bi1));
        assert_eq!(Some(BoardPiece::BlackKing), buffer.get_piece(bi2));
        buffer.set_piece(bi0, None);
        assert_eq!(1, count_pieces(&buffer));
        assert_eq!(None, buffer.get_piece(bi0));
        assert_eq!(None, buffer.get_piece(bi1));
        assert_eq!(Some(BoardPiece::BlackKing), buffer.get_piece(bi2));
        buffer.set_piece(bi2, None);
        assert_eq!(0, count_pieces(&buffer));
        assert_eq!(None, buffer.get_piece(bi0));
        assert_eq!(None, buffer.get_piece(bi1));
        assert_eq!(None, buffer.get_piece(bi2));
    }
}