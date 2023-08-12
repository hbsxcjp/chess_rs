#![allow(dead_code)]
#![allow(non_upper_case_globals)]

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
// use std::rc::Rc;
// use std::rc::Weak;
use num_enum::TryFromPrimitive;
use std::rc::Rc;

pub type Pieces = [piece::Piece; coord::SEATCOUNT];

#[derive(TryFromPrimitive, PartialEq)]
#[repr(usize)]
pub enum MoveDir {
    Back,
    Parallel,
    Forward,
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
            result.push_str(&"_".repeat(ch.to_digit(10).unwrap() as usize));
        }
    }
    // println!("piece_chars: {result}");
    assert_eq!(result.len(), coord::SEATCOUNT);

    result
}

fn piece_chars_to_pieces(piece_chars: &str) -> Pieces {
    let mut result = [piece::Piece::None; coord::SEATCOUNT];
    assert_eq!(piece_chars.len(), coord::SEATCOUNT);

    for (index, ch) in piece_chars.chars().enumerate() {
        result[index] = piece::Piece::new(ch);
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

pub fn fen_to_change(fen: &str, ct: ChangeType) -> String {
    let get_line_vec = |reverse: bool| {
        let mut line_vec: Vec<&str> = fen.split(FENSPLITCHAR).collect();
        if reverse {
            line_vec.reverse();
        }

        line_vec
    };

    fn reverse_line_string(line_vec: Vec<&str>) -> Vec<String> {
        line_vec
            .iter()
            .map(|&line| {
                let mut new_line = String::new();
                for ch in line.chars().rev() {
                    new_line.push(ch);
                }
                new_line
            })
            .collect()
    }

    let sep = &FENSPLITCHAR.to_string();
    match ct {
        ChangeType::Exchange => {
            String::from_iter(fen.chars().into_iter().map(|ch| piece::other_ch(ch)))
        }
        ChangeType::Rotate => reverse_line_string(get_line_vec(true)).join(sep),
        ChangeType::SymmetryH => reverse_line_string(get_line_vec(false)).join(sep),
        ChangeType::SymmetryV => get_line_vec(true).join(sep),
        _ => String::from(fen),
    }
}

pub fn get_bottom_color(pieces: &Pieces) -> piece::Color {
    for index in bit_constant::get_kind_put_indexs(piece::Kind::King, true) {
        if let piece::Piece::Some(color, piece::Kind::King) = pieces[index] {
            return color;
        }
    }

    assert!(false, "没有找到将帅棋子。");
    piece::Color::Red
}

impl Board {
    pub fn new() -> Self {
        Self::from(FEN)
    }

    pub fn from(fen: &str) -> Self {
        Board {
            pieces: fen_to_pieces(fen),
        }
    }

    pub fn get_fen(&self) -> String {
        piece_chars_to_fen(&pieces_to_piece_chars(&self.pieces))
    }

    pub fn bit_board(&self) -> bit_board::BitBoard {
        bit_board::BitBoard::from(&self.pieces)
    }

    pub fn do_move(&mut self, amove: &Rc<amove::Move>) -> piece::Piece {
        let (from_index, to_index) = amove.coordpair.from_to_index();
        let to_piece = self.pieces[to_index];
        self.pieces[to_index] = self.pieces[from_index];
        self.pieces[from_index] = piece::Piece::None;

        to_piece
    }

    pub fn undo_move(&mut self, amove: &Rc<amove::Move>, to_piece: piece::Piece) {
        let (from_index, to_index) = amove.coordpair.from_to_index();

        self.pieces[from_index] = self.pieces[to_index];
        self.pieces[to_index] = to_piece;
    }

    pub fn to_move(&self, amove: &Rc<amove::Move>, contains_self: bool) -> Self {
        let mut board = self.clone();
        for amove in amove.before_moves(contains_self) {
            board.do_move(&amove);
        }

        board
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
                let pieces = self.pieces;
                for (index, &piece) in pieces.iter().enumerate() {
                    self.pieces[Coord::index_to_change(index, ct).unwrap()] = piece;
                }
            }
        }
    }

    pub fn get_zhstr_from_coordpair(&self, coordpair: &CoordPair) -> String {
        let mut result = String::new();
        let from_coord = coordpair.from_coord;
        let piece = &self.pieces[from_coord.index()];
        if let piece::Piece::Some(color, kind) = *piece {
            let to_coord = coordpair.to_coord;
            let from_row = from_coord.row;
            let from_col = from_coord.col;
            let to_row = to_coord.row;
            let to_col = to_coord.col;
            let row_is_same = from_row == to_row;
            let color_is_bottom = color == get_bottom_color(&self.pieces);
            let mut live_coords = self.get_coords_from_color_kind_col(color, kind, from_col);

            result.push(piece.name());
            if live_coords.len() > 1 && kind as usize > piece::Kind::Bishop as usize {
                // 该列有多兵时，需检查多列多兵的情况
                if kind == piece::Kind::Pawn {
                    live_coords = self.get_coords_from_color_multi_pawn(color);
                }

                Self::sort_coords(&mut live_coords, color_is_bottom);
                let pre_chars = Self::get_pre_chars(live_coords.len());
                let index = live_coords
                    .iter()
                    .position(|&coord| coord.row == from_row && coord.col == from_col)
                    .unwrap();
                result.insert(0, pre_chars[index]);
            } else {
                //将帅, 仕(士),相(象): 不用“前”和“后”区别，因为能退的一定在前，能进的一定在后
                let side_col = Coord::get_side_col(from_col, color_is_bottom);
                result.push(Self::get_col_ch(color, side_col));
            }

            result.push(Self::get_move_ch(
                row_is_same,
                color_is_bottom == (to_row < from_row),
            ));
            let num_or_col = if !row_is_same && piece::is_line_move(kind) {
                ((from_row as isize - to_row as isize).abs() - 1) as usize
            } else {
                Coord::get_side_col(to_col, color_is_bottom)
            };
            result.push(Self::get_col_ch(color, num_or_col));
        }
        assert_eq!(self.get_coordpair_from_zhstr(&result), coordpair.clone());

        result
    }

    pub fn get_coordpair_from_zhstr(&self, zhstr: &str) -> CoordPair {
        let zh_chs: Vec<char> = zhstr.chars().collect();
        assert_eq!(zh_chs.len(), 4);

        let color = Self::get_color(zh_chs[3]);
        let color_is_bottom = color == get_bottom_color(&self.pieces);
        let mut index = 0;
        let move_dir = Self::get_move_dir(zh_chs[2]);
        let abs_row_sub = (move_dir == MoveDir::Forward) == color_is_bottom;

        let mut live_coords: Vec<Coord>;
        let mut kind = piece::kind_from_name(zh_chs[0]);
        if kind != piece::Kind::NoKind {
            let col = Self::get_col(color, zh_chs[1]);
            let from_col = Coord::get_side_col(col, color_is_bottom);
            live_coords = self.get_coords_from_color_kind_col(color, kind, from_col);
            assert!(
                live_coords.len() > 0,
                "board:{self:?}\ncolor:{color:?} kind:{kind:?} from_col:{from_col}"
            );

            // 士、象同列时不分前后，以进、退区分棋子位置
            if live_coords.len() == 2 && move_dir == MoveDir::Forward {
                index = 1;
            }
        } else {
            kind = piece::kind_from_name(zh_chs[1]);
            live_coords = if kind == piece::Kind::Pawn {
                self.get_coords_from_color_multi_pawn(color)
            } else {
                self.get_coords_from_color_kind(color, kind)
            };
            assert!(live_coords.len() > 1);

            let pre_chars = Self::get_pre_chars(live_coords.len());
            index = pre_chars.iter().position(|&ch| ch == zh_chs[0]).unwrap();
        }
        assert!(live_coords.len() > index);

        Self::sort_coords(&mut live_coords, color_is_bottom);
        let from_coord = live_coords[index];
        let mut to_row = from_coord.row;
        let col = Self::get_col(color, zh_chs[3]);
        let mut to_col = Coord::get_side_col(col, color_is_bottom);
        if piece::is_line_move(kind) {
            if move_dir != MoveDir::Parallel {
                to_col = from_coord.col;
                if abs_row_sub {
                    to_row -= col + 1;
                } else {
                    to_row += col + 1;
                }
            }
        } else {
            // 斜线走子：仕、相、马
            let col_away = (to_col as isize - from_coord.col as isize).abs() as usize;
            //  相距1或2列
            let row_inc = if kind == piece::Kind::Advisor || kind == piece::Kind::Bishop {
                col_away
            } else {
                if col_away == 1 {
                    2
                } else {
                    1
                }
            };

            if abs_row_sub {
                to_row -= row_inc;
            } else {
                to_row += row_inc;
            }
        }

        let to_coord = Coord::from(to_row, to_col).unwrap();
        CoordPair::from(from_coord, to_coord)
    }

    fn get_coords_from_color_kind(&self, color: piece::Color, kind: piece::Kind) -> Vec<Coord> {
        let mut result = Vec::new();
        for (index, piece) in self.pieces.iter().enumerate() {
            if piece::Piece::Some(color, kind) == *piece {
                result.push(Coord::from_index(index).unwrap());
            }
        }

        result
    }

    fn get_coords_from_color_kind_col(
        &self,
        color: piece::Color,
        kind: piece::Kind,
        col: usize,
    ) -> Vec<Coord> {
        let mut result = self.get_coords_from_color_kind(color, kind);
        result.retain(|&coord| coord.col == col);

        result
    }

    fn get_coords_from_color_multi_pawn(&self, color: piece::Color) -> Vec<Coord> {
        let mut coord_map: HashMap<usize, Vec<Coord>> = HashMap::new();
        for coord in self.get_coords_from_color_kind(color, piece::Kind::Pawn) {
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

    fn sort_coords(coords: &mut Vec<Coord>, color_is_bottom: bool) {
        coords.sort_by(|acoord, bcoord| {
            let mut comp = acoord.col.cmp(&bcoord.col);
            if comp == Ordering::Equal {
                comp = acoord.row.cmp(&bcoord.row).reverse();
            }

            if color_is_bottom {
                comp.reverse()
            } else {
                comp
            }
        });
    }

    fn get_col_ch(color: piece::Color, col: usize) -> char {
        NUMCHARS[color as usize][col]
    }

    fn get_col(color: piece::Color, col_char: char) -> usize {
        NUMCHARS[color as usize]
            .iter()
            .position(|&ch| ch == col_char)
            .unwrap()
    }

    fn get_color(num_ch: char) -> piece::Color {
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

    fn get_move_ch(row_is_same: bool, is_go: bool) -> char {
        MOVECHARS[if row_is_same {
            1
        } else {
            if is_go {
                2
            } else {
                0
            }
        }]
    }

    fn get_move_dir(move_ch: char) -> MoveDir {
        MoveDir::try_from_primitive(MOVECHARS.iter().position(|&ch| ch == move_ch).unwrap())
            .unwrap()
    }

    pub fn get_pgnzh_pattern() -> String {
        format!(
            "{}|{}",
            Self::get_pgnzh_pattern_color(piece::Color::Red),
            Self::get_pgnzh_pattern_color(piece::Color::Black)
        )
    }

    pub fn get_pgnzh_pattern_color(color: piece::Color) -> String {
        let mut name_chars = String::new();
        for ch in piece::NAMECHARS[color as usize] {
            name_chars.push(ch);
        }
        let mut num_chars = String::new();
        for ch in NUMCHARS[color as usize] {
            num_chars.push(ch);
        }
        let mut pos_chars = String::new();
        for ch in POSCHARS {
            pos_chars.push(ch);
        }
        let mut move_chars = String::new();
        for ch in MOVECHARS {
            move_chars.push(ch);
        }

        format!(
            "[{}{}{}]{{2}}[{}][{}]",
            name_chars, num_chars, pos_chars, move_chars, num_chars
        )
    }

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
    use crate::common;

    use super::*;

    #[test]
    fn test_board() {
        for (fen, piece_chars, to_string) in common::FEN_PIECES_CHARS {
            assert_eq!(fen_to_piece_chars(fen), piece_chars);
            assert_eq!(piece_chars_to_fen(piece_chars), fen);

            let mut board = Board::from(fen);

            assert_eq!(board.to_string(), to_string);
            // let name = fen.split_at(3).0;
            // std::fs::write(format!("tests/output/board_{name}.txt"), board.to_string())
            //     .expect("Write Err.");
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
