#![allow(dead_code)]

use crate::board_constant::*;
use crate::piece::*;

pub struct Board {
    // 基本数据
    // colors: [Color; SEATCOUNT],
    // kinds: [Kind; SEATCOUNT],
    pieces: [[BitBoard; KINDCOUNT]; COLORCOUNT],

    // 计算中间存储数据(基本局面改动时更新)
    color_pieces: [BitBoard; COLORCOUNT],
    all_pieces: BitBoard,
    rotate_pieces: BitBoard,

    // 哈希局面数据
    hashkey: u64,
    // private static HistoryRecord? historyRecord;
}

const FEN: &str = "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR";

const FENSPLITCHAR: char = '/';

const NUMCHARS: [[char; COLCOUNT]; COLORCOUNT] = [
    ['一', '二', '三', '四', '五', '六', '七', '八', '九'],
    ['１', '２', '３', '４', '５', '６', '７', '８', '９'],
];

const POSCHARS: [char; 3] = ['前', '中', '后'];

const MOVECHARS: [char; 3] = ['退', '平', '进'];

fn piece_chars_to_fen(piece_chars: &str) -> String {
    let mut result = String::new();
    //  Regex.Replace(
    //     string.Join(FENSplitChar,
    //         Enumerable.Range(0, Coord.RowCount).Select(row => pieceChars.Substring(row * Coord.ColCount, Coord.ColCount))),
    //     $"{Piece.NullCh}+",
    //     match => match.Value.Length.ToString());

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

impl Board {
    pub fn new(fen: &str) -> Board {
        let mut board: Board = Board {
            pieces: [[0; KINDCOUNT]; COLORCOUNT],
            color_pieces: [0; COLORCOUNT],
            all_pieces: 0,
            rotate_pieces: 0,
            hashkey: 0,
        };

        board
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_board() {
        let result = fen_to_piece_chars(FEN);
        assert_eq!(result, "rnbakabnr__________c_____c_p_p_p_p_p__________________P_P_P_P_P_C_____C__________RNBAKABNR");

        let result = fen_to_piece_chars("5a3/4ak2r/6R2/8p/9/9/9/B4N2B/4K4/3c5");
        assert_eq!(result, "_____a_______ak__r______R__________p___________________________B____N__B____K_______c_____");

        let result = fen_to_piece_chars("5k3/9/9/9/9/9/4rp3/2R1C4/4K4/9");
        assert_eq!(result, "_____k____________________________________________________rp_____R_C________K_____________");
    }
}
