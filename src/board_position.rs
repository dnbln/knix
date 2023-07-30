use std::fmt;
use std::fmt::Formatter;
use std::ops::{Add, Neg};
use std::str::FromStr;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum BoardColumn {
    A = 1,
    B = 2,
    C = 3,
    D = 4,
    E = 5,
    F = 6,
    G = 7,
    H = 8,
}

impl fmt::Display for BoardColumn {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.char())
    }
}

impl BoardColumn {
    pub fn char(self) -> char {
        match self {
            Self::A => 'A',
            Self::B => 'B',
            Self::C => 'C',
            Self::D => 'D',
            Self::E => 'E',
            Self::F => 'F',
            Self::G => 'G',
            Self::H => 'H',
        }
    }

    pub fn try_from_char(c: char) -> Option<Self> {
        let col = match c {
            'A' | 'a' => Self::A,
            'B' | 'b' => Self::B,
            'C' | 'c' => Self::C,
            'D' | 'd' => Self::D,
            'E' | 'e' => Self::E,
            'F' | 'f' => Self::F,
            'G' | 'g' => Self::G,
            'H' | 'h' => Self::H,
            _ => return None,
        };

        Some(col)
    }

    #[track_caller]
    pub fn from_char(c: char) -> Self {
        Self::try_from_char(c).expect("Invalid column")
    }

    /// index: between 1 and 8
    pub unsafe fn from_index_unchecked(index: u8) -> Self {
        std::mem::transmute::<u8, Self>(index)
    }

    pub fn from_index(index: u8) -> Option<Self> {
        if !(1..=8).contains(&index) {
            return None;
        }

        Some(unsafe { Self::from_index_unchecked(index) })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoardPosition {
    row: u8,
    column: BoardColumn,
}

impl fmt::Display for BoardPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { row, column } = self;
        write!(f, "{column}{row}")
    }
}

/// Encodes a board position in a single u8.
#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct BoardIndex {
    // number between 0 and 63
    pos: u8,
}

impl fmt::Debug for BoardIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} (@{})", BoardPosition::from_index(*self), self.pos)
    }
}

impl fmt::Display for BoardIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BoardPosition::from_index(*self))
    }
}

impl BoardIndex {
    pub fn rank(self) -> u8 {
        self.pos / 8 + 1
    }

    pub fn file(self) -> u8 {
        self.pos % 8 + 1
    }

    pub fn get_pos(self) -> u8 {
        self.pos
    }
}

impl BoardIndex {
    pub unsafe fn new_unchecked(pos: u8) -> Self {
        Self { pos }
    }

    pub fn new(pos: u8) -> Option<Self> {
        if !(0..64).contains(&pos) {
            return None;
        }

        Some(unsafe { Self::new_unchecked(pos) })
    }

    pub unsafe fn unchecked_add(self, rhs: BoardIndexDelta) -> BoardIndex {
        Self::new_unchecked((self.pos as i8 + 8 * rhs.delta_rank + rhs.delta_file) as u8)
    }

    pub fn checked_add(self, rhs: BoardIndexDelta) -> Option<BoardIndex> {
        if rhs.delta_rank < 0 && self.rank() <= rhs.delta_rank.neg() as u8 {
            return None;
        }
        if rhs.delta_rank > 0 && self.rank() > (8 - rhs.delta_rank) as u8 {
            return None;
        }
        if rhs.delta_file < 0 && self.file() <= rhs.delta_file.neg() as u8 {
            return None;
        }
        if rhs.delta_file > 0 && self.file() > (8 - rhs.delta_file) as u8 {
            return None;
        }

        Some(unsafe { self.unchecked_add(rhs) })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct BoardIndexDelta {
    delta_rank: i8,
    delta_file: i8,
}

impl BoardIndexDelta {
    pub const fn new(delta_rank: i8, delta_file: i8) -> Self {
        Self {
            delta_rank,
            delta_file,
        }
    }

    pub const fn delta_file(delta_file: i8) -> Self {
        Self::new(0, delta_file)
    }

    pub const fn delta_rank(delta_rank: i8) -> Self {
        Self::new(delta_rank, 0)
    }
}

impl Add<BoardIndexDelta> for BoardIndex {
    type Output = BoardIndex;

    #[track_caller]
    fn add(self, rhs: BoardIndexDelta) -> Self::Output {
        self.checked_add(rhs).expect("Invalid position after delta")
    }
}

impl BoardPosition {
    pub unsafe fn new_unchecked(row: u8, column: BoardColumn) -> Self {
        Self { row, column }
    }

    pub fn try_new(row: u8, column: BoardColumn) -> Option<Self> {
        if !(1..=8).contains(&row) {
            return None;
        }

        Some(unsafe { Self::new_unchecked(row, column) })
    }

    #[track_caller]
    pub fn new(row: u8, column: BoardColumn) -> Self {
        Self::try_new(row, column).expect("Invalid position")
    }

    pub fn from_index(index: BoardIndex) -> Self {
        let row = index.rank();
        let column = index.file();
        Self {
            row,
            column: unsafe { BoardColumn::from_index_unchecked(column) },
        }
    }

    pub fn to_index(self) -> BoardIndex {
        let idx = (self.row - 1) * 8 + (self.column as u8 - 1);
        debug_assert!(idx < 64);
        unsafe { BoardIndex::new_unchecked(idx) }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum InvalidBoardPosition {
    NotLength2,
    FirstNotValidColumn,
    SecondNotValidRow,
}

impl FromStr for BoardPosition {
    type Err = InvalidBoardPosition;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.chars();
        let c1 = iter.next().ok_or(InvalidBoardPosition::NotLength2)?;
        let c2 = iter.next().ok_or(InvalidBoardPosition::NotLength2)?;
        if iter.next().is_some() {
            return Err(InvalidBoardPosition::NotLength2);
        }

        let col =
            BoardColumn::try_from_char(c1).ok_or(InvalidBoardPosition::FirstNotValidColumn)?;
        if !('1'..='8').contains(&c2) {
            return Err(InvalidBoardPosition::SecondNotValidRow);
        }
        let row = c2 as u8 - b'0';
        Ok(unsafe { BoardPosition::new_unchecked(row, col) })
    }
}
