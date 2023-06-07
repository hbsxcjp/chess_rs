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
    pub fn new(fen: &str) -> Self {
        ManualMove {
            board: board::Board::new(fen),

            root_move: amove::Move::root(),
            current_move: amove::Move::root(),
        }
    }

    pub fn set_from(
        &mut self,
        fen: &str,
        byte_vec: &Vec<u8>,
        version: u8,
        keyxyf: usize,
        keyxyt: usize,
        keyrmksize: usize,
        f32keys: &[u8],
    ) {
        let __sub = |a, b| (a as usize - b as usize) as u8; // 保持为<256

        let mut pos: usize = 1024;
        let mut __readBytes = |size| {
            // int pos = (int)stream.Position;
            // stream.Read(bytes, 0, size);
            let mut bytes = vec![0; size];
            bytes.copy_from_slice(&byte_vec[pos..(pos + size)]);

            if version > 10
            // '字节解密'
            {
                for index in 0..size {
                    bytes[index] = __sub(bytes[index], f32keys[(pos + index) % 32]);
                }
            }

            pos += size;
            bytes
        };

        let mut __getRemarksize = || {
            const INTSIZE: usize = 4;
            let data = __readBytes(INTSIZE);

            (data[0] as usize
                + (data[1] << 8) as usize
                + (data[2] << 16) as usize
                + (data[3] << 24) as usize)
                - keyrmksize
        };

        // Encoding codec = Encoding.GetEncoding("gb2312"); // "gb2312"
        // let mut data =[0u32;4];
        // let frc = data[0];
        // let trc = data[1];
        // let tag = data[2];
        let mut __readDataAndGetRemark = || {
            const DATASIZE: usize = 4;
            let mut data = __readBytes(DATASIZE);
            let mut RemarkSize = 0;
            //    let frc = &data[0];
            //     let trc = &data[1];
            if version <= 10 {
                data[2] = (if data[2] & 0xF0 != 0 { 0x80 } else { 0 })
                    | (if data[2] & 0x0F != 0 { 0x40 } else { 0 });
                RemarkSize = __getRemarksize();
            } else {
                data[2] &= 0xE0;
                if data[2] & 0x20 != 0 {
                    RemarkSize = __getRemarksize();
                }
            }

            let mut remark = String::new();
            // # 有注解
            if RemarkSize > 0 {
                let remark_vec = __readBytes(RemarkSize);
                remark = GBK.decode(&remark_vec, DecoderTrap::Ignore).unwrap();
            }
            // var remark = codec.GetString(rem).Replace('\0', ' ')
            // .Replace("\r\n", "\n").Trim();
            // return remark.Length > 0 ? remark : null;

            (data, remark)
        };

        // stream.Seek(1024, SeekOrigin.Begin);
        let (data, remark) = __readDataAndGetRemark();
        *self.root_move.remark.borrow_mut() = remark;
        // self.root_move.remark = __readDataAndGetRemark()?.Trim();

        if data[2] & 0x80 == 0
        // 无左子树
        {
            return;
        }

        // 有左子树
        let mut beforeMoves = vec![];
        beforeMoves.push(self.root_move.clone());
        let mut isOther = false;
        let mut beforeMove = self.root_move.clone();
        // 当前棋子为根，且有后继棋子时，表明深度搜索已经回退到根，已经没有后续棋子了
        while !(beforeMove.is_root() && beforeMove.after.borrow().len() > 0) {
            let (data, remark) = __readDataAndGetRemark();
            // let remark = __readDataAndGetRemark();
            //# 一步棋的起点和终点有简单的加密计算，读入时需要还原

            let fcolrow = __sub(data[0], 0x18 + keyxyf as u8);
            let tcolrow = __sub(data[1], 0x20 + keyxyt as u8);
            if fcolrow > 89 || tcolrow > 89 {
                assert!(false, "fcolrow > 89 || tcolrow > 89 ? ");
            }

            let frow = 10 - 1 - fcolrow % 10;
            let fcol = fcolrow / 10;
            let trow = 10 - 1 - tcolrow % 10;
            let tcol = tcolrow / 10; //

            let coordPair =
                CoordPair::from_rowcol(frow as usize, fcol as usize, trow as usize, tcol as usize)
                    .unwrap();
            let tag = data[2];
            let hasNext = (tag & 0x80) != 0;
            let hasOther = (tag & 0x40) != 0;

            if beforeMove.coordpair == coordPair {
                // let message=format!("Error: {:?} {}", coordPair, beforeMove.remark.borrow());
                assert!(false, "Error.");
            } else {
                if isOther {
                    beforeMove = beforeMove.before.upgrade().unwrap();
                }

                beforeMove = beforeMove.add(coordPair, remark);
                if hasNext && hasOther {
                    beforeMoves.push(beforeMove);
                }

                isOther = !hasNext;
                if isOther && !hasOther && beforeMoves.len() > 0 {
                    beforeMove = beforeMoves.pop().unwrap(); // 最后时，将回退到根
                }
            }
        }

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
        let manual_move = ManualMove::new("5a3/4ak2r/6R2/8p/9/9/9/B4N2B/4K4/3c5");

        assert_eq!("[(0,0)->(0,0)] \n", manual_move.to_string());
    }
}
