#![allow(dead_code)]

use crate::coord::CoordPair;
use crate::{amove, coord};
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use std::collections::VecDeque;
// use crate::bit_constant;
// use std::borrow::Borrow;
use crate::board;
use crate::utility;
// use crate::manual;
// use std::cell::RefCell;
use std::rc::Rc;
// use std::rc::Weak;
use regex;

#[derive(Debug)]
pub struct ManualMove {
    pub board: board::Board,
    pub root_move: Rc<amove::Move>,
}

impl ManualMove {
    fn from(fen: &str, root_move: Rc<amove::Move>) -> Self {
        ManualMove {
            board: board::Board::new(fen),
            root_move,
        }
    }

    pub fn new() -> Self {
        ManualMove::from(board::FEN, amove::Move::root())
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
        *root_move.remark.borrow_mut() = remark;

        if data[2] & 0x80 != 0 {
            let mut before_moves = vec![root_move.clone()];
            let mut before_move = root_move.clone();
            let mut is_other = false;
            // 当前棋子非根，或为根尚无后续棋子/当前棋子为根，且有后继棋子时，表明深度搜索已经回退到根，已经没有后续棋子了
            while pos < input.len()
                && (!before_move.is_root() || before_move.after.borrow().len() == 0)
            {
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
                let coord_pair = CoordPair::from_rowcol(frow, fcol, trow, tcol).unwrap();
                let tag = data[2];
                let has_next = (tag & 0x80) != 0;
                let has_other = (tag & 0x40) != 0;
                if before_move.coordpair == coord_pair {
                    assert!(false, "Error.");
                }

                if is_other {
                    before_move = before_move.before.upgrade().unwrap();
                }

                before_move = before_move.add(coord_pair, remark);
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
        fn take_remark_after_num(input: &mut &[u8]) -> (String, usize) {
            let remark = utility::read_string(input);
            let after_num = utility::read_be_u32(input) as usize;

            (remark, after_num)
        }

        let root_move = amove::Move::root();
        let (root_remark, root_after_num) = take_remark_after_num(input);
        *root_move.remark.borrow_mut() = root_remark;

        let mut move_after_num_deque: VecDeque<(Rc<amove::Move>, usize)> = VecDeque::new();
        move_after_num_deque.push_back((root_move.clone(), root_after_num));
        while move_after_num_deque.len() > 0 {
            let (before_move, before_after_num) = move_after_num_deque.pop_front().unwrap();
            for _ in 0..before_after_num {
                let (rowcol_bytes, rest) = input.split_at(4);
                *input = rest;

                let frow = rowcol_bytes[0] as usize;
                let fcol = rowcol_bytes[1] as usize;
                let trow = rowcol_bytes[2] as usize;
                let tcol = rowcol_bytes[3] as usize;
                let coordpair = CoordPair::from_rowcol(frow, fcol, trow, tcol).unwrap();
                let (remark, after_num) = take_remark_after_num(input);

                let amove = before_move.add(coordpair, remark);
                if after_num > 0 {
                    move_after_num_deque.push_back((amove, after_num));
                }
            }
        }

        ManualMove::from(fen, root_move)
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        fn append_remark_after_num(result: &mut Vec<u8>, amove: &Rc<amove::Move>) {
            utility::write_string(result, amove.remark.borrow().as_str());
            utility::write_be_u32(result, amove.after.borrow().len() as u32);
        }

        let mut result = Vec::new();
        append_remark_after_num(&mut result, &self.root_move);
        for amove in self.get_all_after_moves() {
            let (frow, fcol, trow, tcol) = amove.coordpair.row_col();
            result.append(&mut vec![frow as u8, fcol as u8, trow as u8, tcol as u8]);
            append_remark_after_num(&mut result, &amove);
        }

        result
    }

    fn get_all_after_moves(&self) -> Vec<Rc<amove::Move>> {
        fn enqueue_after(move_deque: &mut VecDeque<Rc<amove::Move>>, amove: &Rc<amove::Move>) {
            for after_move in amove.after.borrow().iter() {
                move_deque.push_back(after_move.clone());
            }
        }

        let mut all_after_moves: Vec<Rc<amove::Move>> = Vec::new();
        let mut move_deque: VecDeque<Rc<amove::Move>> = VecDeque::new();
        enqueue_after(&mut move_deque, &self.root_move);
        let mut id = 0;
        while let Some(amove) = move_deque.pop_front() {
            id += 1;
            *amove.id.borrow_mut() = id;

            enqueue_after(&mut move_deque, &amove);
            all_after_moves.push(amove);
        }

        all_after_moves
    }

    pub fn to_string(&self, record_type: coord::RecordType) -> String {
        let mut reslut = self.root_move.to_string(record_type);
        for amove in self.get_all_after_moves() {
            match record_type {
                coord::RecordType::PgnZh => {
                    let board = self.board.to_move(&amove.before.upgrade().unwrap());
                    reslut.push_str(&amove.to_string_pgnzh(board));
                }
                _ => {
                    reslut.push_str(&amove.to_string(record_type));
                }
            }
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

        assert_eq!("", manual_move.to_string(coord::RecordType::Txt));
    }
}
