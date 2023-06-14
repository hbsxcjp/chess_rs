#![allow(dead_code)]

use crate::amove;
use crate::bit_board;
use crate::bit_constant;
use crate::coord::Coord;
use crate::coord::CoordPair;
use crate::coord::{self, ChangeType};
use crate::piece;
use std::cmp::Ordering;
use std::collections::HashMap;
// use std::cell::RefCell;
use std::rc::Rc;
// use std::rc::Weak;

pub type Pieces = [piece::Piece; coord::SEATCOUNT];

#[derive(Debug)]
pub struct Board {
    pieces: Pieces,
}

pub const FEN: &str = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR";

const FENSPLITCHAR: char = '/';

const NUMCHARS: [[char; coord::COLCOUNT]; piece::COLORCOUNT] = [
    ['一', '二', '三', '四', '五', '六', '七', '八', '九'],
    ['１', '２', '３', '４', '５', '６', '７', '８', '９'],
];

const POSCHARS: [char; 3] = ['前', '中', '后'];

const MOVECHARS: [char; 3] = ['退', '平', '进'];

pub fn piece_chars_to_fen(piece_chars: &str) -> String {
    fn push_num_str(result: &mut String, null_num: &mut i32) {
        if *null_num > 0 {
            result.push_str(&null_num.to_string());
            *null_num = 0;
        };
    }

    let mut result = String::new();
    let mut null_num = 0;
    for (index, ch) in piece_chars.chars().enumerate() {
        if ch.is_ascii_alphabetic() {
            push_num_str(&mut result, &mut null_num);
            result.push(ch);
        } else {
            null_num += 1;
        }

        if (index + 1) % coord::COLCOUNT == 0 {
            push_num_str(&mut result, &mut null_num);
            if index < coord::SEATCOUNT - 1 {
                result.push(FENSPLITCHAR);
            }
        }
    }

    result
}

fn fen_to_piece_chars(fen: &str) -> String {
    let mut result = String::new();
    for ch in fen.chars() {
        if ch.is_ascii_alphabetic() {
            result.push(ch);
        } else if ch.is_ascii_digit() {
            result.push_str(&"_".repeat(ch.to_digit(10).unwrap_or_default() as usize));
        }
    }

    result
}

fn piece_chars_to_pieces(piece_chars: &str) -> Pieces {
    let mut result = [piece::Piece::None; coord::SEATCOUNT];
    for (index, ch) in piece_chars.chars().enumerate() {
        result[index] = piece::Piece::new(ch);
        if index + 1 == result.len() {
            break;
        }
    }

    result
}

fn pieces_to_piece_chars(pieces: &Pieces) -> String {
    let mut result = String::new();
    for piece in pieces {
        result.push(piece.ch());
    }

    result
}

pub fn fen_to_pieces(fen: &str) -> Pieces {
    piece_chars_to_pieces(&fen_to_piece_chars(fen))
}

fn pieces_to_fen(pieces: &Pieces) -> String {
    piece_chars_to_fen(&pieces_to_piece_chars(pieces))
}

pub fn fen_to_change(fen: &str, ct: ChangeType) -> String {
    let get_line_vec = |reverse: bool| {
        let mut line_vec: Vec<&str> = fen.split(FENSPLITCHAR).collect();
        if reverse {
            line_vec.reverse();
        }

        line_vec
    };

    let rev_line_vec_string = |line_vec: Vec<&str>| {
        let mut new_line_vec = Vec::new();
        for line in line_vec {
            let mut new_line = String::new();
            for ch in line.chars() {
                new_line.insert(0, ch);
            }
            new_line_vec.push(new_line);
        }

        new_line_vec
    };

    match ct {
        ChangeType::Exchange => {
            String::from_iter(fen.chars().into_iter().map(|ch| piece::other_ch(ch)))
        }
        ChangeType::Rotate => {
            rev_line_vec_string(get_line_vec(true)).join(&FENSPLITCHAR.to_string())
        }
        ChangeType::SymmetryH => {
            rev_line_vec_string(get_line_vec(false)).join(&FENSPLITCHAR.to_string())
        }
        ChangeType::SymmetryV => get_line_vec(true).join(&FENSPLITCHAR.to_string()),
        _ => String::from(fen),
    }
}

pub fn get_bottom_color(pieces: &Pieces) -> piece::Color {
    for index in bit_constant::get_kind_put_indexs(piece::Kind::King, true) {
        if let piece::Piece::Some(color, piece::Kind::King) = pieces[index] {
            return color;
        }
    }

    piece::Color::Red
}

impl Board {
    pub fn new(fen: &str) -> Self {
        Board {
            pieces: fen_to_pieces(fen),
        }
    }

    pub fn get_fen(&self) -> String {
        pieces_to_fen(&self.pieces)
    }

    pub fn bit_board(&self) -> bit_board::BitBoard {
        bit_board::BitBoard::new(&self.pieces)
    }

    pub fn to_move(&self, amove: &Rc<amove::Move>) -> Self {
        let mut pieces = self.pieces.clone();
        for bmove in amove.before_moves() {
            let from_index = bmove.coordpair.from_coord.index();
            let to_index = bmove.coordpair.to_coord.index();
            pieces[to_index] = pieces[from_index];
            pieces[from_index] = piece::Piece::None;
        }

        Board { pieces }
    }

    pub fn to_change(&mut self, ct: ChangeType) {
        match ct {
            ChangeType::NoChange => (),
            ChangeType::Exchange => {
                for piece in &mut self.pieces {
                    if let piece::Piece::Some(color, _) = piece {
                        *color = piece::other_color(*color);
                    }
                }
            }
            _ => {
                let pieces = self.pieces.clone();
                for (index, _) in pieces.iter().enumerate() {
                    if let Some(coord) = Coord::from_index(index) {
                        self.pieces[index] = pieces[coord.to_change(ct).index()];
                    }
                }
            }
        }
    }

    pub fn get_zhstr_from_coordpair(&self, coordpair: CoordPair) -> String {
        let result = String::new();

        result
    }

    pub fn get_coordpair_from_zhstr(&self, zhstr: String) -> CoordPair {
        let result = CoordPair::new();

        result
    }

    fn get_rows_from_color_kind(&self, color: piece::Color, kind: piece::Kind) -> Vec<Coord> {
        let mut result = Vec::new();
        for (index, piece) in self.pieces.iter().enumerate() {
            match piece {
                piece::Piece::Some(acolor, akind) if *acolor == color && *akind == kind => {
                    result.push(Coord::from_index(index).unwrap());
                }
                _ => (),
            }
        }

        result
    }

    fn get_rows_from_color_kind_col(
        &self,
        color: piece::Color,
        kind: piece::Kind,
        col: usize,
    ) -> Vec<Coord> {
        let mut result = self.get_rows_from_color_kind(color, kind);
        result.retain(|&coord| coord.col == col);

        result
    }

    fn get_rows_from_color_multi_pawn(&self, color: piece::Color) -> Vec<Coord> {
        let mut coord_map: HashMap<usize, Vec<Coord>> = HashMap::new();
        for coord in self.get_rows_from_color_kind(color, piece::Kind::Pawn) {
            let col = coord.col;
            if coord_map.contains_key(&col) {
                coord_map.get_mut(&col).unwrap().push(coord);
            } else {
                coord_map.insert(col, vec![coord]);
            }
        }

        let mut result = Vec::new();
        for coords in coord_map.values_mut() {
            if coords.len() > 1 {
                result.append(coords);
            }
        }

        result
    }

    fn sort_coordss(coords: &mut Vec<Coord>, color_is_bottom: bool) {
        coords.sort_by(|acoord, bcoord| {
            let mut comp = acoord.col.cmp(&bcoord.col);
            if comp == Ordering::Equal {
                comp = acoord.col.cmp(&bcoord.col);
            }

            if color_is_bottom {
                if comp == Ordering::Less {
                    Ordering::Greater
                } else {
                    Ordering::Less
                }
            } else {
                comp
            }
        });
    }

    fn get_col_char(color: piece::Color, col: usize) -> char {
        NUMCHARS[color as usize][col]
    }

    fn get_col(color: piece::Color, col_char: char) -> usize {
        NUMCHARS[color as usize].binary_search(&col_char).unwrap()
    }

    fn get_num_from_ch(num_ch: char) -> piece::Color {
        if NUMCHARS[piece::Color::Red as usize].contains(&num_ch) {
            piece::Color::Red
        } else {
            piece::Color::Black
        }
    }

    fn get_pre_chars(count: usize) -> Vec<char> {
        match count {
            2 => vec![POSCHARS[0], POSCHARS[2]],
            3 => POSCHARS.to_vec(),
            _ => NUMCHARS[piece::Color::Red as usize][0..5].to_vec(),
        }
    }

    fn get_move_ch(is_same_row: bool, is_go: bool) -> char {
        MOVECHARS[if is_same_row {
            1
        } else {
            if is_go {
                2
            } else {
                0
            }
        }]
    }

    fn get_move_dir(move_ch: char) -> isize {
        MOVECHARS.binary_search(&move_ch).unwrap() as isize - 1
    }

    // fn get_pgnzh_char_pattern(color: piece::Color) -> String {
    //     format!("[{}{}{}]{{2}}[{}][{}]",
    //     piece::NA
    //      NUMCHARS)
    // }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (index, piece) in self.pieces.iter().enumerate() {
            result.push(piece.print_name());

            if (index + 1) % coord::COLCOUNT == 0 {
                result.push('\n');
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board() {
        let fen_piece_chars=[
            (FEN,
            "rnbakabnr__________c_____c_p_p_p_p_p__________________P_P_P_P_P_C_____C__________RNBAKABNR",
        "車馬象士将士象馬車
－－－－－－－－－
－砲－－－－－砲－
卒－卒－卒－卒－卒
－－－－－－－－－
－－－－－－－－－
兵－兵－兵－兵－兵
－炮－－－－－炮－
－－－－－－－－－
车马相仕帅仕相马车
"),
            ("5a3/4ak2r/6R2/8p/9/9/9/B4N2B/4K4/3c5",
            "_____a_______ak__r______R__________p___________________________B____N__B____K_______c_____",
        "－－－－－士－－－
－－－－士将－－車
－－－－－－车－－
－－－－－－－－卒
－－－－－－－－－
－－－－－－－－－
－－－－－－－－－
相－－－－马－－相
－－－－帅－－－－
－－－砲－－－－－
"),
            ("2b1kab2/4a4/4c4/9/9/3R5/9/1C7/4r4/2BK2B2",
            "__b_kab______a________c_________________________R_______________C___________r______BK__B__",
        "－－象－将士象－－
－－－－士－－－－
－－－－砲－－－－
－－－－－－－－－
－－－－－－－－－
－－－车－－－－－
－－－－－－－－－
－炮－－－－－－－
－－－－車－－－－
－－相帅－－相－－
"),
            ("4kab2/4a4/4b4/3N5/9/4N4/4n4/4B4/4A4/3AK1B2",
            "____kab______a________b_______N__________________N________n________B________A_______AK_B__",
        "－－－－将士象－－
－－－－士－－－－
－－－－象－－－－
－－－马－－－－－
－－－－－－－－－
－－－－马－－－－
－－－－馬－－－－
－－－－相－－－－
－－－－仕－－－－
－－－仕帅－相－－
"),
        ];

        for (fen, piece_chars, to_string) in fen_piece_chars {
            assert_eq!(fen_to_piece_chars(fen), piece_chars);
            assert_eq!(piece_chars_to_fen(piece_chars), fen);

            let mut board = Board::new(fen);

            assert_eq!(board.to_string(), to_string);
            // let name = fen.split_at(3).0;
            // std::fs::write(format!("tests/{name}.txt"), board.to_string()).expect("Write Err.");
            // dbg!(board);

            for ct in [
                ChangeType::Exchange,
                ChangeType::Rotate,
                ChangeType::SymmetryH,
                ChangeType::SymmetryV,
                ChangeType::NoChange,
            ] {
                let fen = fen_to_change(&board.get_fen(), ct);
                board.to_change(ct);
                assert_eq!(fen, board.get_fen());
            }
        }
    }
}
