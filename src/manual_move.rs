#![allow(dead_code)]

use crate::amove;
use crate::bit_constant::CoordPair;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
// use crate::bit_constant;
// use std::borrow::Borrow;
use crate::board;
// use std::cell::RefCell;
use std::rc::Rc;
// use std::rc::Weak;

#[derive(Debug)]
pub struct ManualMove {
    pub board: board::Board,

    pub root_move: Rc<amove::Move>,
    pub current_move: Rc<amove::Move>,
}

impl ManualMove {
    fn from(fen: &str, root_move: Rc<amove::Move>) -> Self {
        let current_move = root_move.clone();
        ManualMove {
            board: board::Board::new(fen),
            root_move,
            current_move,
        }
    }

    pub fn new() -> Self {
        ManualMove::from(board::FEN, amove::Move::root())
    }

    pub fn from_xqf(
        fen: &str,
        byte_vec: &Vec<u8>,
        version: u8,
        keyxyf: usize,
        keyxyt: usize,
        keyrmksize: usize,
        f32keys: &[u8],
    ) -> Self {
        let __sub = |a, b| (a as isize - b as isize) as u8; // 保持为<256

        let read_bytes = |pos: &mut usize, size| {
            let new_pos = *pos + size;
            let mut bytes = byte_vec[*pos..new_pos].to_vec();
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
            const INTSIZE: usize = 4;
            let data = read_bytes(pos, INTSIZE);
            (data[0] as usize
                + ((data[1] as usize) << 8)
                + ((data[2] as usize) << 16)
                + ((data[3] as usize) << 24))
                - keyrmksize
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

            (
                data,
                if remark_size > 0 {
                    GBK.decode(&read_bytes(pos, remark_size), DecoderTrap::Ignore)
                        .unwrap()
                        .replace("\r\n", "\n")
                        .trim()
                        .into()
                } else {
                    String::new()
                },
            )
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
            while pos < byte_vec.len()
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

    pub fn to_string(&self) -> String {
        self.root_move.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_move() {
        let manual_move = ManualMove::new();

        assert_eq!("", manual_move.to_string());
    }
}
