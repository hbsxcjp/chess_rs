#![allow(dead_code)]

use crate::board;
use crate::coord::CoordPair;
use crate::evaluation;
use crate::{amove, common, coord};
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use std::collections::VecDeque;
use std::rc::Rc;
// use crate::bit_constant;
// use std::borrow::Borrow;
// use crate::utility;
// use crate::manual;
// use std::cell::RefCell;
// use std::rc::Weak;
// use regex;

#[derive(Debug)]
pub struct ManualMove {
    board: board::Board,
    root_move: Rc<amove::Move>,
}

impl PartialEq for ManualMove {
    fn eq(&self, other: &Self) -> bool {
        self.board == other.board
            && self.to_string(coord::RecordType::Txt) == other.to_string(coord::RecordType::Txt)
    }
}

impl ManualMove {
    pub fn new() -> Self {
        ManualMove::from(board::FEN, amove::Move::root())
    }

    fn from(fen: &str, root_move: Rc<amove::Move>) -> Self {
        ManualMove {
            board: board::Board::from(fen),
            root_move,
        }
    }

    pub fn from_rowcols(fen: &str, rowcols: &str) -> common::Result<Self> {
        let root_move = amove::Move::root();
        let mut amove = root_move.clone();
        let coordpairs = Self::get_coordpairs_from_rowcols(rowcols)?;
        for coordpair in coordpairs {
            amove = amove.append(coordpair, String::new());
        }

        Ok(ManualMove::from(fen, root_move))
    }

    pub fn from_xqf(
        fen: &str,
        input: &Vec<u8>,
        version: u8,
        keyxyf: usize,
        keyxyt: usize,
        keyrmksize: usize,
        f32keys: &[u8],
    ) -> common::Result<Self> {
        let __sub = |a, b| (a as isize - b as isize) as u8; // 保持为<256

        let read_bytes = |pos: &mut usize, size| {
            let new_pos = *pos + size;
            let mut bytes = input[*pos..new_pos].to_vec();
            if version > 10 {
                // '字节解密'
                for (index, abyte) in bytes.iter_mut().enumerate() {
                    *abyte = __sub(*abyte, f32keys[(*pos + index) % 32]);
                }
            }

            *pos = new_pos;
            bytes
        };

        let get_remark_size = |pos: &mut usize| {
            let data = read_bytes(pos, std::mem::size_of::<u32>());
            u32::from_le_bytes(data.try_into().unwrap()) as usize - keyrmksize
        };

        let get_data_remark = |pos: &mut usize| {
            const DATASIZE: usize = 4;
            let mut data = read_bytes(pos, DATASIZE);
            let mut remark_size = 0;
            if version <= 10 {
                data[2] = (if data[2] & 0xF0 != 0 { 0x80 } else { 0 })
                    | (if data[2] & 0x0F != 0 { 0x40 } else { 0 });
                remark_size = get_remark_size(pos);
            } else {
                data[2] &= 0xE0;
                if data[2] & 0x20 != 0 {
                    remark_size = get_remark_size(pos);
                }
            }

            let remark = if remark_size > 0 {
                GBK.decode(&read_bytes(pos, remark_size), DecoderTrap::Ignore)
                    .unwrap()
                    .replace("\r\n", "\n")
                    .trim()
                    .into()
            } else {
                String::new()
            };

            (data, remark)
        };

        let mut pos: usize = 1024;
        let root_move = amove::Move::root();
        let (data, remark) = get_data_remark(&mut pos);
        root_move.set_remark(remark);

        if data[2] & 0x80 != 0 {
            let mut before_moves = vec![root_move.clone()];
            let mut before_move = root_move.clone();
            let mut is_other = false;
            // 当前棋子非根，或为根尚无后续棋子/当前棋子为根，且有后继棋子时，表明深度搜索已经回退到根，已经没有后续棋子了
            while pos < input.len() && (!before_move.is_root() || before_move.after_len() == 0) {
                let (data, remark) = get_data_remark(&mut pos);
                //# 一步棋的起点和终点有简单的加密计算，读入时需要还原
                let fcolrow = __sub(data[0], (0x18 + keyxyf as usize) as u8);
                let tcolrow = __sub(data[1], (0x20 + keyxyt as usize) as u8);
                if fcolrow > 89 || tcolrow > 89 {
                    // assert!(false, "fcolrow > 89 || tcolrow > 89 ? ");
                    return Err(common::GenerateError::IndexOut);
                }

                let frow = (10 - 1 - fcolrow % 10) as usize;
                let fcol = (fcolrow / 10) as usize;
                let trow = (10 - 1 - tcolrow % 10) as usize;
                let tcol = (tcolrow / 10) as usize;
                let coord_pair = CoordPair::from_row_col(frow, fcol, trow, tcol).unwrap();
                let tag = data[2];
                let has_next = (tag & 0x80) != 0;
                let has_other = (tag & 0x40) != 0;
                if before_move.coordpair == coord_pair {
                    // assert!(false, "before_move.coordpair == coord_pair? Error.");
                    // return Err(common::GenerateError::IndexOut);
                    continue;
                }

                if is_other {
                    if let Some(before) = before_move.before() {
                        before_move = before;
                    }
                }

                before_move = before_move.append(coord_pair, remark);
                if has_next && has_other {
                    before_moves.push(before_move.clone());
                }

                is_other = !has_next;
                if is_other && !has_other && before_moves.len() > 0 {
                    // 最后时，将回退到根
                    before_move = before_moves.pop().unwrap();
                }
            }
        }

        // List<Move> allMoves = RootMove.AllAfterMoves;
        // allMoves.Insert(0, RootMove);
        // allMoves.ForEach(move
        //     => move.AfterMoves?.RemoveAll(move
        //         => !GetBoardWith(move.Before).BitBoard.CanMove(move.CoordPair)));
        let board = board::Board::from(fen);
        for amove in &root_move.get_all_after_moves() {
            let (from_index, to_index) = amove.coordpair.from_to_index();
            let is_valid = board
                .to_move(amove, false)
                .bit_board()
                .is_valid(from_index, to_index);
            assert!(is_valid, "({from_index}, {to_index}) is invalid!");
        }
        
        Ok(ManualMove::from(fen, root_move))
    }

    pub fn from_bin(fen: &str, input: &mut &[u8]) -> Self {
        let root_move = amove::Move::root();
        let remark = common::read_string(input);
        let after_num = common::read_be_u32(input) as usize;
        root_move.set_remark(remark);

        let mut move_after_num_deque: VecDeque<(Rc<amove::Move>, usize)> = VecDeque::new();
        move_after_num_deque.push_back((root_move.clone(), after_num));
        while move_after_num_deque.len() > 0 {
            let (before_move, before_after_num) = move_after_num_deque.pop_front().unwrap();
            for _ in 0..before_after_num {
                let coordpair = common::read_coordpair(input);
                let remark = common::read_string(input);
                let after_num = common::read_be_u32(input) as usize;

                let amove = before_move.append(coordpair, remark);
                if after_num > 0 {
                    move_after_num_deque.push_back((amove, after_num));
                }
            }
        }

        ManualMove::from(fen, root_move)
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();

        common::write_string(&mut result, &self.root_move.remark());
        common::write_be_u32(&mut result, self.root_move.after_len() as u32);
        for amove in self.root_move.get_all_after_moves() {
            common::write_coordpair(&mut result, &amove.coordpair);

            common::write_string(&mut result, &amove.remark());
            common::write_be_u32(&mut result, amove.after_len() as u32);
        }

        result
    }

    pub fn from_string(
        fen: &str,
        manual_move_str: &str,
        record_type: coord::RecordType,
    ) -> common::Result<Self> {
        let pgnzh_pattern = board::Board::get_pgnzh_pattern();
        let pgn_pattern = match record_type {
            coord::RecordType::PgnRc => r"\d{4}",
            coord::RecordType::PgnIccs => r"(?:[A-I]\d){2}",
            coord::RecordType::PgnZh => pgnzh_pattern.as_str(),
            _ => r"(?:\(\d,\d\)){2}",
        };
        let remark_num_pattern = r"(?:\{([\s\S]+?)\})?(?:\((\d+)\))?\n";
        let amove_pattern = format!("({pgn_pattern}){remark_num_pattern}");
        let root_move_re = regex::Regex::new(&("^".to_string() + remark_num_pattern)).unwrap();
        let amove_re = regex::Regex::new(&amove_pattern).unwrap();
        // println!("{}\n{}", manual_move_str, remark_num_pattern);

        let board = board::Board::from(fen);
        let root_move = amove::Move::root();
        if let Some(root_caps) = root_move_re.captures(manual_move_str) {
            if let Some(remark) = root_caps.at(1) {
                root_move.set_remark(remark.to_string());
            }

            if let Some(after_num_str) = root_caps.at(2) {
                if let Ok(root_after_num) = after_num_str.parse() {
                    let mut move_after_num_deque: VecDeque<(Rc<amove::Move>, usize)> =
                        VecDeque::new();
                    move_after_num_deque.push_back((root_move.clone(), root_after_num));
                    let mut caps_iter = amove_re.captures_iter(manual_move_str);
                    while move_after_num_deque.len() > 0 {
                        let (before_move, before_after_num) =
                            move_after_num_deque.pop_front().unwrap();
                        let the_board = board.to_move(&before_move, true);
                        for _ in 0..before_after_num {
                            let caps =
                                caps_iter.next().ok_or(common::GenerateError::StringParse)?;
                            let coordpair_str = caps.at(1).unwrap();
                            let coordpair = match record_type {
                                coord::RecordType::PgnZh => {
                                    // println!("{:?}\nfen:{}", the_board, fen);
                                    // println!("{} ", coordpair_str);
                                    the_board.get_coordpair_from_zhstr(coordpair_str)
                                }
                                _ => CoordPair::from_string(coordpair_str, record_type)?,
                            };
                            let remark = if let Some(remark) = caps.at(2) {
                                remark.to_string()
                            } else {
                                String::new()
                            };
                            let after_num: usize = if let Some(after_num_str) = caps.at(3) {
                                after_num_str.parse().unwrap_or(0)
                            } else {
                                0
                            };

                            let amove = before_move.append(coordpair, remark);
                            if after_num > 0 {
                                move_after_num_deque.push_back((amove, after_num));
                            }
                        }
                    }
                }
            }
        }

        Ok(ManualMove {
            board,
            root_move: root_move,
        })
    }

    pub fn get_zorbist(&self) -> evaluation::Zorbist {
        let mut zorbist = evaluation::Zorbist::new();
        for amove in self.root_move.get_all_after_moves() {
            let mut bit_board = self.board.to_move(&amove, false).bit_board();
            if let Some((key, aspect)) = bit_board.get_key_asp_amove(&amove) {
                zorbist.insert(key, aspect);
            }
        }

        zorbist
    }

    pub fn get_rowcols(&self) -> String {
        let mut reslut = String::new();
        let mut amove = self.root_move.clone();
        while let Some(after) = amove.after() {
            let bmove = after.first().unwrap();
            reslut.push_str(&bmove.coordpair.to_string(coord::RecordType::PgnRc));
            amove = bmove.clone();
        }

        reslut
    }

    pub fn get_coordpairs_from_rowcols(rowcols: &str) -> common::Result<Vec<coord::CoordPair>> {
        let mut coordpairs = vec![];
        for index in 0..(rowcols.len() / 4) {
            let coordpair = CoordPair::from_string(
                &rowcols[index * 4..(index + 1) * 4],
                coord::RecordType::PgnRc,
            )?;
            coordpairs.push(coordpair);
        }

        Ok(coordpairs)
    }

    pub fn to_string(&self, record_type: coord::RecordType) -> String {
        let mut reslut = self.root_move.to_string(record_type, &self.board);
        for amove in self.root_move.get_all_after_moves() {
            reslut.push_str(&amove.to_string(record_type, &self.board));
        }

        reslut
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_move() {
        let manual_move = ManualMove::new();

        assert_eq!("\n", manual_move.to_string(coord::RecordType::Txt));
    }
}
