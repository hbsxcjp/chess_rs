#![allow(dead_code)]

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
    PgnRc,
    PgnIccs,
    PgnZh,
}

impl RecordType {
    pub fn get_ext_name(ext_type: RecordType) -> String {
        format!("{:?}", ext_type).to_ascii_lowercase()
    }

    pub fn get_ext_type(ext_name: &str) -> RecordType {
        match ext_name {
            _ if ext_name == Self::get_ext_name(Self::Xqf) => Self::Xqf,
            _ if ext_name == Self::get_ext_name(Self::Txt) => Self::Txt,
            _ if ext_name == Self::get_ext_name(Self::PgnRc) => Self::PgnRc,
            _ if ext_name == Self::get_ext_name(Self::PgnIccs) => Self::PgnIccs,
            _ => Self::PgnZh,
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

    pub fn from_index(index: usize) -> Option<Self> {
        if index < SEATCOUNT {
            Some(Coord {
                row: index / COLCOUNT,
                col: index % COLCOUNT,
            })
        } else {
            None
        }
    }

    pub fn index(&self) -> usize {
        self.row * COLCOUNT + self.col
    }

    pub fn from_rowcol(row: usize, col: usize) -> Option<Self> {
        if row < ROWCOUNT && col < COLCOUNT {
            Some(Coord { row, col })
        } else {
            None
        }
    }

    pub fn row_col(&self) -> (usize, usize) {
        (self.row, self.col)
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

    pub fn index_to_change(index: usize, ct: ChangeType) -> usize {
        if let Some(coord) = Coord::from_index(index) {
            coord.to_change(ct).index()
        } else {
            usize::MAX
        }
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
            RecordType::PgnRc => format!("{}{}", self.row, self.col),
            RecordType::PgnIccs => format!(
                "{}{}",
                char::from_u32('A' as u32 + self.col as u32).unwrap_or('X'),
                self.row
            ),

            // RecordType::Txt => format!("({},{})", self.row, self.col),
            // _ => String::new(),
            _ => format!("({},{})", self.row, self.col),
        }
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

    pub fn from_rowcol(frow: usize, fcol: usize, trow: usize, tcol: usize) -> Option<Self> {
        if let Some(from_coord) = Coord::from_rowcol(frow, fcol) {
            if let Some(to_coord) = Coord::from_rowcol(trow, tcol) {
                return Some(CoordPair::from(from_coord, to_coord));
            }
        }

        None
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
