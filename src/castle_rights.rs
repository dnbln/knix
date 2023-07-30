use std::ops::{BitAnd, BitOr, BitOrAssign};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct CastleRights {
    rights: u8,
}

impl CastleRights {
    pub fn revoke(&mut self, revoked: CastleRights) {
        self.rights &= !revoked.rights;
    }
}

impl BitOr<CastleRights> for CastleRights {
    type Output = CastleRights;

    fn bitor(self, rhs: CastleRights) -> Self::Output {
        CastleRights {
            rights: self.rights | rhs.rights,
        }
    }
}

impl BitOrAssign<CastleRights> for CastleRights {
    fn bitor_assign(&mut self, rhs: CastleRights) {
        self.rights |= rhs.rights;
    }
}

impl BitAnd<CastleRights> for CastleRights {
    type Output = CastleRights;

    fn bitand(self, rhs: CastleRights) -> Self::Output {
        CastleRights {
            rights: self.rights & rhs.rights,
        }
    }
}

impl CastleRights {
    pub const EMPTY: CastleRights = CastleRights { rights: 0 };
    pub const WHITE_KING_SIDE: CastleRights = CastleRights { rights: 1 };
    pub const WHITE_QUEEN_SIDE: CastleRights = CastleRights { rights: 2 };
    pub const BLACK_KING_SIDE: CastleRights = CastleRights { rights: 4 };
    pub const BLACK_QUEEN_SIDE: CastleRights = CastleRights { rights: 8 };

    pub fn right_from_fen_char(c: char) -> Option<CastleRights> {
        match c {
            'K' => Some(CastleRights::WHITE_KING_SIDE),
            'Q' => Some(CastleRights::WHITE_QUEEN_SIDE),
            'k' => Some(CastleRights::BLACK_KING_SIDE),
            'q' => Some(CastleRights::BLACK_QUEEN_SIDE),
            _ => None,
        }
    }

    pub fn rights_from_fen_str(s: &str) -> Result<CastleRights, InvalidCastleRight> {
        let mut total = CastleRights::EMPTY;

        if s == "-" {
            return Ok(total);
        }

        for c in s.chars() {
            let right = Self::right_from_fen_char(c).ok_or(InvalidCastleRight(c))?;
            total |= right;
        }

        Ok(total)
    }

    pub fn has_rights(&self, k: CastleRights) -> bool {
        (*self & k).rights == k.rights
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, thiserror::Error)]
#[error("char {0} is an invalid castle right")]
pub struct InvalidCastleRight(char);
