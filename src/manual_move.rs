#![allow(dead_code)]

use crate::coord::CoordPair;
use crate::{amove, common, coord};
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use std::collections::VecDeque;
// use crate::bit_constant;
// use std::borrow::Borrow;
use crate::board;
// use crate::utility;
// use crate::manual;
// use std::cell::RefCell;
use std::rc::Rc;
// use std::rc::Weak;
// use regex;

#[derive(Debug)]
pub struct ManualMove {
    board: board::Board,
    root_move: Rc<amove::Move>,
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

    pub fn from_xqf(
        fen: &str,
        input: &Vec<u8>,
        version: u8,
        keyxyf: usize,
        keyxyt: usize,
        keyrmksize: usize,
        f32keys: &[u8],
    ) -> Self {
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
            while pos < input.len() && (!before_move.is_root() || before_move.after().is_empty()) {
                let (data, remark) = get_data_remark(&mut pos);
                //# 一步棋的起点和终点有简单的加密计算，读入时需要还原
                let fcolrow = __sub(data[0], (0x18 + keyxyf as usize) as u8);
                let tcolrow = __sub(data[1], (0x20 + keyxyt as usize) as u8);
                if fcolrow > 89 || tcolrow > 89 {
                    assert!(false, "fcolrow > 89 || tcolrow > 89 ? ");
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
                    assert!(false, "Error.");
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

        ManualMove::from(fen, root_move)
        // List<Move> allMoves = RootMove.AllAfterMoves;
        // allMoves.Insert(0, RootMove);
        // allMoves.ForEach(move
        //     => move.AfterMoves?.RemoveAll(move
        //         => !GetBoardWith(move.Before).BitBoard.CanMove(move.CoordPair)));
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
        common::write_be_u32(&mut result, self.root_move.after().len() as u32);
        for amove in self.get_all_after_moves() {
            common::write_coordpair(&mut result, &amove.coordpair);

            common::write_string(&mut result, &amove.remark());
            common::write_be_u32(&mut result, amove.after().len() as u32);
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
                        for _ in 0..before_after_num {
                            let caps = caps_iter.next().ok_or(common::ParseError::StringParse)?;
                            let coordpair =
                                CoordPair::from_string(caps.at(1).unwrap(), record_type)?;
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

        Ok(ManualMove::from(fen, root_move))
    }

    fn get_all_after_moves(&self) -> Vec<Rc<amove::Move>> {
        fn enqueue_after(move_deque: &mut VecDeque<Rc<amove::Move>>, amove: &Rc<amove::Move>) {
            for bmove in amove.after() {
                move_deque.push_back(bmove.clone());
            }
        }

        let mut all_after_moves: Vec<Rc<amove::Move>> = Vec::new();
        let mut move_deque: VecDeque<Rc<amove::Move>> = VecDeque::new();
        enqueue_after(&mut move_deque, &self.root_move);
        while let Some(amove) = move_deque.pop_front() {
            enqueue_after(&mut move_deque, &amove);
            all_after_moves.push(amove);
        }

        all_after_moves
    }

    pub fn to_string(&self, record_type: coord::RecordType) -> String {
        let mut reslut = self.root_move.to_string(record_type, &self.board);
        for amove in self.get_all_after_moves() {
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
