#![allow(dead_code)]
use crate::common;
use std::fmt::{Display, Formatter}; //coord,

pub const ROWCOUNT: usize = 10;
pub const COLCOUNT: usize = 9;
pub const SEATCOUNT: usize = ROWCOUNT * COLCOUNT;

pub const SIDECOUNT: usize = 2;
pub const LEGSTATECOUNT: usize = 1 << 4;
pub const COLSTATECOUNT: usize = 1 << ROWCOUNT;
pub const ROWSTATECOUNT: usize = 1 << COLCOUNT;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RecordType {
    Xqf,
    Bin,
    Txt,
    PgnIccs,
    PgnRc,
    PgnZh,
}

impl RecordType {
    pub fn ext_name(&self) -> String {
        format!("{:?}", self).to_ascii_lowercase()
    }

    pub fn get_record_type(file_name: &str) -> common::Result<RecordType> {
        let ext_pos = file_name
            .rfind('.')
            .ok_or(common::ParseError::RecordTypeError)?;
        let ext_name = file_name
            .get(ext_pos + 1..)
            .ok_or(common::ParseError::RecordTypeError)?;

        match ext_name {
            _ if ext_name == RecordType::Xqf.ext_name() => Ok(RecordType::Xqf),
            _ if ext_name == RecordType::Bin.ext_name() => Ok(RecordType::Bin),
            _ if ext_name == RecordType::Txt.ext_name() => Ok(RecordType::Txt),
            _ if ext_name == RecordType::PgnIccs.ext_name() => Ok(RecordType::PgnIccs),
            _ if ext_name == RecordType::PgnRc.ext_name() => Ok(RecordType::PgnRc),
            _ if ext_name == RecordType::PgnZh.ext_name() => Ok(RecordType::PgnZh),
            _ => Err(common::ParseError::RecordTypeError),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ChangeType {
    Exchange,
    Rotate,
    SymmetryH,
    SymmetryV,
    NoChange,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coord {
    pub row: usize,
    pub col: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CoordPair {
    pub from_coord: Coord,
    pub to_coord: Coord,
}

impl Coord {
    pub fn new() -> Self {
        Coord { row: 0, col: 0 }
    }

    pub fn from(row: usize, col: usize) -> common::Result<Self> {
        if row >= ROWCOUNT {
            Err(common::ParseError::RowOut)
        } else if col >= COLCOUNT {
            Err(common::ParseError::ColOut)
        } else {
            Ok(Coord { row, col })
        }
    }

    pub fn from_index(index: usize) -> common::Result<Self> {
        Self::from(index / COLCOUNT, index % COLCOUNT)
    }

    pub fn from_string(coord_str: &str, record_type: RecordType) -> common::Result<Self> {
        match record_type {
            RecordType::PgnRc => {
                let row_col = coord_str
                    .parse::<usize>()
                    .map_err(|_| common::ParseError::StringParse)?;

                Self::from(row_col / 10, row_col % 10)
            }
            RecordType::PgnIccs => {
                let row = coord_str
                    .get(1..2)
                    .ok_or(common::ParseError::StringParse)?
                    .parse()
                    .map_err(|_| common::ParseError::StringParse)?;
                let col_ch = coord_str
                    .chars()
                    .next()
                    .ok_or(common::ParseError::StringParse)?;
                let col = col_ch as usize - 'A' as usize;

                Self::from(row, col)
            }
            RecordType::Txt => {
                let row = coord_str
                    .get(1..2)
                    .ok_or(common::ParseError::StringParse)?
                    .parse()
                    .map_err(|_| common::ParseError::StringParse)?;
                let col = coord_str
                    .get(3..4)
                    .ok_or(common::ParseError::StringParse)?
                    .parse()
                    .map_err(|_| common::ParseError::StringParse)?;

                Self::from(row, col)
            }
            _ => Err(common::ParseError::StringParse),
        }
    }

    pub fn index(&self) -> usize {
        self.row * COLCOUNT + self.col
    }

    pub fn to_change(&self, ct: ChangeType) -> Self {
        match ct {
            ChangeType::Rotate => Self {
                row: Self::symmetry_row(self.row),
                col: Self::symmetry_col(self.col),
            },
            ChangeType::SymmetryH => Self {
                row: self.row,
                col: Self::symmetry_col(self.col),
            },
            ChangeType::SymmetryV => Self {
                row: Self::symmetry_row(self.row),
                col: self.col,
            },
            _ => *self,
        }
    }

    pub fn index_to_change(index: usize, ct: ChangeType) -> Option<usize> {
        let coord = Coord::from_index(index).ok()?;

        Some(coord.to_change(ct).index())
    }

    pub fn get_side_col(col: usize, color_is_bottom: bool) -> usize {
        if color_is_bottom {
            Coord::symmetry_col(col)
        } else {
            col
        }
    }

    fn symmetry_row(row: usize) -> usize {
        ROWCOUNT - 1 - row
    }

    fn symmetry_col(col: usize) -> usize {
        COLCOUNT - 1 - col
    }

    pub fn to_string(&self, record_type: RecordType) -> String {
        match record_type {
            // RecordType::Txt => format!("({},{})", self.row, self.col),
            RecordType::Txt => format!("{}", self),
            RecordType::PgnIccs => format!(
                "{}{}",
                char::from_u32('A' as u32 + self.col as u32).unwrap(),
                self.row
            ),
            RecordType::PgnRc => format!("{}{}", self.row, self.col),

            _ => String::new(),
        }
    }
}

impl Display for Coord {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "({},{})", self.row, self.col)
    }
}

impl CoordPair {
    pub fn new() -> Self {
        CoordPair {
            from_coord: Coord::new(),
            to_coord: Coord::new(),
        }
    }

    pub fn from(from_coord: Coord, to_coord: Coord) -> Self {
        CoordPair {
            from_coord,
            to_coord,
        }
    }

    pub fn from_row_col(
        frow: usize,
        fcol: usize,
        trow: usize,
        tcol: usize,
    ) -> common::Result<Self> {
        let from_coord = Coord::from(frow, fcol)?;
        let to_coord = Coord::from(trow, tcol)?;

        Ok(CoordPair::from(from_coord, to_coord))
    }

    pub fn from_string(coordpair_str: &str, record_type: RecordType) -> common::Result<Self> {
        let mid = coordpair_str.len() / 2;
        let from_coord = Coord::from_string(&coordpair_str[..mid], record_type)?;
        let to_coord = Coord::from_string(&coordpair_str[mid..], record_type)?;

        Ok(CoordPair::from(from_coord, to_coord))
    }

    pub fn from_to_index(&self) -> (usize, usize) {
        (self.from_coord.index(), self.to_coord.index())
    }

    pub fn row_col(&self) -> (usize, usize, usize, usize) {
        (
            self.from_coord.row,
            self.from_coord.col,
            self.to_coord.row,
            self.to_coord.col,
        )
    }

    pub fn to_string(&self, record_type: RecordType) -> String {
        format!(
            "{}{}",
            self.from_coord.to_string(record_type),
            self.to_coord.to_string(record_type)
        )
    }
}
