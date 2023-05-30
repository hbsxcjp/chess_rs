#![allow(dead_code)]

use num_enum::TryFromPrimitive;
use crate::bit_constant;

#[derive(Clone, Copy, Debug, TryFromPrimitive, PartialEq)]
#[repr(usize)]
pub enum Color {
    Red,
    Black,
    NoColor,
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

#[derive(Clone, Copy, Debug)]
pub enum Piece {
    None,
    Some(Color, Kind),
}

const NULLCHAR: char = '_';

const KINGCHAR: char = 'k';
const ADVISORCHAR: char = 'a';
const BISHOPCHAR: char = 'b';
const KNIGHTCHAR: char = 'n';
const ROOKCHAR: char = 'r';
const CANNONCHAR: char = 'c';
const PAWNCHAR: char = 'p';

const NULLNAME: char = '－';

const REDKINGNAME: char = '帅';
const REDADVISORNAME: char = '仕';
const REDBISHOPNAME: char = '相';
const KNIGHTNAME: char = '马';
const ROOKNAME: char = '车';
const CANNONNAME: char = '炮';
const REDPAWNNAME: char = '兵';

const BLACKKINGNAME: char = '将';
const BLACKADVISORNAME: char = '士';
const BLACKBISHOPNAME: char = '象';
const BLACKKNIGHTNAME: char = '馬';
const BLACKROOKNAME: char = '車';
const BLACKCANNONNAME: char = '砲';
const BLACKPAWNNAME: char = '卒';

pub const COLORARRAY: [Color; bit_constant::COLORCOUNT] = [Color::Red, Color::Black];
pub const KINDARRAY: [Kind; bit_constant::KINDCOUNT] = [
    Kind::King,
    Kind::Advisor,
    Kind::Bishop,
    Kind::Knight,
    Kind::Rook,
    Kind::Cannon,
    Kind::Pawn,
];

pub fn is_line_move(kind: &Kind) -> bool {
    match kind {
        Kind::King => true,
        Kind::Rook => true,
        Kind::Cannon => true,
        Kind::Pawn => true,
        _ => false,
    }
}

pub fn other_color(color: Color) -> Color {
    match color {
        Color::Red => Color::Black,
        Color::Black => Color::Red,
        Color::NoColor => Color::NoColor,
    }
}

fn color(ch: char) -> Color {
    match ch.is_ascii_uppercase() {
        true => Color::Red,
        false => Color::Black,
    }
}

fn kind(ch: char) -> Kind {
    match ch.to_ascii_lowercase() {
        KINGCHAR => Kind::King,
        ADVISORCHAR => Kind::Advisor,
        BISHOPCHAR => Kind::Bishop,
        KNIGHTCHAR => Kind::Knight,
        ROOKCHAR => Kind::Rook,
        CANNONCHAR => Kind::Cannon,
        _ => Kind::Pawn,
    }
}

pub fn get_ch(color: &Color, kind: &Kind) -> char {
    let ch = match kind {
        Kind::King => KINGCHAR,
        Kind::Advisor => ADVISORCHAR,
        Kind::Bishop => BISHOPCHAR,
        Kind::Knight => KNIGHTCHAR,
        Kind::Rook => ROOKCHAR,
        Kind::Cannon => CANNONCHAR,
        Kind::Pawn => PAWNCHAR,
        Kind::NoKind => NULLCHAR,
    };

    match color {
        Color::Red => ch.to_ascii_uppercase(),
        _ => ch,
    }
}

impl Piece {
    pub fn new(ch: char) -> Piece {
        match ch != NULLCHAR {
            true => Piece::Some(color(ch), kind(ch)),
            false => Piece::None,
        }
    }

    pub fn ch(&self) -> char {
        match self {
            Self::None => NULLCHAR,
            Self::Some(color, kind) => get_ch(color, kind),
        }
    }

    pub fn name(&self) -> char {
        match self {
            Self::None => NULLNAME,
            Self::Some(color, kind) => match kind {
                Kind::King => match color {
                    Color::Red => REDKINGNAME,
                    _ => BLACKKINGNAME,
                },
                Kind::Advisor => match color {
                    Color::Red => REDADVISORNAME,
                    _ => BLACKADVISORNAME,
                },
                Kind::Bishop => match color {
                    Color::Red => REDBISHOPNAME,
                    _ => BLACKBISHOPNAME,
                },
                Kind::Knight => KNIGHTNAME,
                Kind::Rook => ROOKNAME,
                Kind::Cannon => CANNONNAME,
                Kind::Pawn => match color {
                    Color::Red => REDPAWNNAME,
                    _ => BLACKPAWNNAME,
                },
                Kind::NoKind => NULLNAME,
            },
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
