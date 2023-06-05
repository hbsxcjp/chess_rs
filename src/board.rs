#![allow(dead_code)]

use crate::bit_board;
use crate::bit_constant;
use crate::piece;

pub type Pieces = [piece::Piece; bit_constant::SEATCOUNT];

#[derive(Debug)]
pub struct Board {
    // 基本数据
    pieces: Pieces,
    // 计算棋盘各种状态
    // bottom_color: piece::Color,
    // bit_board: bit_board::BitBoard,
}

pub const FEN: &str = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR";

const FENSPLITCHAR: char = '/';

const NUMCHARS: [[char; bit_constant::COLCOUNT]; bit_constant::COLORCOUNT] = [
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
    let mut index = 0;
    for ch in piece_chars.chars() {
        if ch.is_ascii_alphabetic() {
            push_num_str(&mut result, &mut null_num);
            result.push(ch);
        } else {
            null_num += 1;
        }

        index += 1;
        if index % bit_constant::COLCOUNT == 0 {
            push_num_str(&mut result, &mut null_num);
            if index < bit_constant::SEATCOUNT {
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
    let mut result = [piece::Piece::None; bit_constant::SEATCOUNT];
    let mut index = 0;
    for ch in piece_chars.chars() {
        result[index] = piece::Piece::new(ch);
        index += 1;
        if index == result.len() {
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

pub fn get_bottom_color(pieces: &Pieces) -> piece::Color {
    let mut bottom_color = piece::Color::Red;
    for index in bit_constant::get_kind_put_indexs(piece::Kind::King, true) {
        match pieces[index] {
            piece::Piece::Some(color, piece::Kind::King) => {
                bottom_color = color;
                break;
            }
            _ => (),
        };
    }

    bottom_color
}

impl Board {
    pub fn new(fen: &str) -> Self {
        Board {
            pieces: fen_to_pieces(fen),
            // bottom_color: get_bottom_color(&pieces),
            // bit_board: bit_board::BitBoard::new(&pieces),
        }
    }

    pub fn get_fen(&self) -> String {
        pieces_to_fen(&self.pieces)
    }

    pub fn bit_board(&self) -> bit_board::BitBoard {
        bit_board::BitBoard::new(&self.pieces)
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        let mut index = 0;
        for piece in self.pieces {
            result.push(piece.print_name());

            index += 1;
            if index % bit_constant::COLCOUNT == 0 {
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

            let board = Board::new(fen);
            assert_eq!(board.to_string(), to_string);
            // let name = fen.split_at(3).0;
            // std::fs::write(format!("tests/{name}.txt"), board.to_string()).expect("Write Err.");
            // dbg!(board);
        }
    }
}
