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
    PgnIccs,
    PgnRc,
    PgnZh,
}

impl RecordType {
    pub fn ext_name(&self) -> String {
        format!("{:?}", self).to_ascii_lowercase()
    }

    pub fn get_record_type(file_name: &str) -> RecordType {
        let ext_pos = file_name.rfind('.').unwrap_or(0);
        let ext_name = &file_name[(ext_pos + 1)..];

        match ext_name {
            _ if ext_name == RecordType::Xqf.ext_name() => RecordType::Xqf,
            _ if ext_name == RecordType::Bin.ext_name() => RecordType::Bin,
            _ if ext_name == RecordType::Txt.ext_name() => RecordType::Txt,
            _ if ext_name == RecordType::PgnIccs.ext_name() => RecordType::PgnIccs,
            _ if ext_name == RecordType::PgnRc.ext_name() => RecordType::PgnRc,
            _ if ext_name == RecordType::PgnZh.ext_name() => RecordType::PgnZh,
            _ => RecordType::PgnZh,
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

    pub fn from_rowcol(row: usize, col: usize) -> Option<Self> {
        if row < ROWCOUNT && col < COLCOUNT {
            Some(Coord { row, col })
        } else {
            None
        }
    }

    pub fn from_string(coord_str: &str, record_type: RecordType) -> Option<Self> {
        if ((record_type == RecordType::PgnIccs || record_type == RecordType::PgnRc)
            && coord_str.len() < 2)
            || (record_type == RecordType::PgnZh && coord_str.len() < 4)
        {
            return None;
        }

        match record_type {
            RecordType::PgnRc => Self::from_rowcol(
                coord_str[0..1].parse().unwrap(),
                coord_str[1..2].parse().unwrap(),
            ),
            RecordType::PgnIccs => Self::from_rowcol(
                coord_str[1..2].parse().unwrap(),
                coord_str.chars().next().unwrap() as usize - 'A' as usize,
            ),
            RecordType::Txt => Self::from_rowcol(
                coord_str[1..2].parse().unwrap(),
                coord_str[3..4].parse().unwrap(),
            ),
            _ => None,
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
        if let Some(coord) = Coord::from_index(index) {
            Some(coord.to_change(ct).index())
        } else {
            None
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
            RecordType::Txt => format!("({},{})", self.row, self.col),
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

    pub fn from_string(coordpair_str: &str, record_type: RecordType) -> Self {
        let mid = coordpair_str.len() / 2;
        Self::from(
            Coord::from_string(&coordpair_str[..mid], record_type).unwrap_or(Coord::new()),
            Coord::from_string(&coordpair_str[mid..], record_type).unwrap_or(Coord::new()),
        )
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
