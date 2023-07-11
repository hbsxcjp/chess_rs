#![allow(dead_code)]

// use crate::bit_constant;
use num_enum::TryFromPrimitive;

#[derive(Clone, Copy, Debug, TryFromPrimitive, PartialEq)]
#[repr(usize)]
pub enum Color {
    Red,
    Black,
}

#[derive(Clone, Copy, Debug, TryFromPrimitive, PartialEq)]
#[repr(usize)]
pub enum Kind {
    King,
    Advisor,
    Bishop,
    Knight,
    Rook,
    Cannon,
    Pawn,
    NoKind,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Piece {
    None,
    Some(Color, Kind),
}

pub const COLORCOUNT: usize = 2;
pub const KINDCOUNT: usize = 7;

const NULLCHAR: char = '_';

const NULLNAME: char = '－';

const CHCHARS: [[char; KINDCOUNT]; COLORCOUNT] = [
    ['K', 'A', 'B', 'N', 'R', 'C', 'P'],
    ['k', 'a', 'b', 'n', 'r', 'c', 'p'],
];

pub const NAMECHARS: [[char; KINDCOUNT]; COLORCOUNT] = [
    ['帅', '仕', '相', '马', '车', '炮', '兵'],
    ['将', '士', '象', '马', '车', '炮', '卒'],
];
const BLACKKNIGHTNAME: char = '馬';
const BLACKROOKNAME: char = '車';
const BLACKCANNONNAME: char = '砲';

pub const COLORARRAY: [Color; COLORCOUNT] = [Color::Red, Color::Black];
pub const KINDARRAY: [Kind; KINDCOUNT] = [
    Kind::King,
    Kind::Advisor,
    Kind::Bishop,
    Kind::Knight,
    Kind::Rook,
    Kind::Cannon,
    Kind::Pawn,
];

pub fn other_color(color: Color) -> Color {
    match color {
        Color::Red => Color::Black,
        Color::Black => Color::Red,
    }
}

fn color(ch: char) -> Color {
    if ch.is_ascii_uppercase() {
        Color::Red
    } else {
        Color::Black
    }
}

pub fn kind(ch: char) -> Kind {
    for (index, ach) in CHCHARS[color(ch) as usize].iter().enumerate() {
        if ch == *ach {
            return Kind::try_from_primitive(index).unwrap_or(Kind::NoKind);
        }
    }

    Kind::NoKind
}

pub fn color_from_name(name: char) -> Color {
    if NAMECHARS[Color::Red as usize].contains(&name) {
        Color::Red
    } else {
        Color::Black
    }
}

pub fn kind_from_name(name: char) -> Kind {
    for (index, aname) in NAMECHARS[color_from_name(name) as usize].iter().enumerate() {
        if name == *aname {
            return Kind::try_from_primitive(index).unwrap_or(Kind::NoKind);
        }
    }

    Kind::NoKind
}

pub fn is_line_move(kind: Kind) -> bool {
    match kind {
        Kind::King | Kind::Rook | Kind::Cannon | Kind::Pawn => true,
        _ => false,
    }
}

pub fn other_ch(ch: char) -> char {
    if ch.is_ascii_lowercase() {
        ch.to_ascii_uppercase()
    } else {
        ch.to_ascii_lowercase()
    }
}

pub fn get_ch(color: Option<Color>, kind: Kind) -> char {
    match kind {
        Kind::NoKind => NULLCHAR,
        _ => CHCHARS[color.unwrap() as usize][kind as usize],
    }
}

impl Piece {
    pub fn new(ch: char) -> Piece {
        if ch != NULLCHAR {
            Piece::Some(color(ch), kind(ch))
        } else {
            Piece::None
        }
    }

    pub fn ch(&self) -> char {
        match self {
            Self::None => NULLCHAR,
            Self::Some(color, kind) => get_ch(Some(*color), *kind),
        }
    }

    pub fn name(&self) -> char {
        match self {
            Self::None => NULLNAME,
            Self::Some(color, kind) => NAMECHARS[*color as usize][*kind as usize],
        }
    }

    pub fn print_name(&self) -> char {
        match self {
            Self::None => NULLNAME,
            Self::Some(color, kind) => match color {
                Color::Black => match kind {
                    Kind::Knight => BLACKKNIGHTNAME,
                    Kind::Rook => BLACKROOKNAME,
                    Kind::Cannon => BLACKCANNONNAME,
                    _ => self.name(),
                },
                _ => self.name(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pieces() {
        let piece_chars = "_KABNRCPkabnrcp";
        let piece_names = "－帅仕相马车炮兵将士象马车炮卒";
        let piece_print_names = "－帅仕相马车炮兵将士象馬車砲卒";
        let mut chars_result = String::new();
        let mut names_result = String::new();
        let mut print_names_result = String::new();

        for ch in piece_chars.chars() {
            let piece = Piece::new(ch);
            chars_result.push(piece.ch());
            names_result.push(piece.name());
            print_names_result.push(piece.print_name());
        }

        assert_eq!(chars_result, piece_chars);
        assert_eq!(names_result, piece_names);
        assert_eq!(print_names_result, piece_print_names);
    }
}
