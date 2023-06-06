#![allow(dead_code)]

use crate::amove;
use crate::bit_constant::CoordPair;
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
        keyxyf: u8,
        keyxyt: u8,
        keyrmksize: u32,
        f32keys: &[u8],
    ) {
        let __sub= |a, b| { (a as usize - b as usize) as u8 }; // 保持为<256

        let __readBytes= |bytes, size|
        {
            int pos = (int)stream.Position;
            stream.Read(bytes, 0, size);
            if (xQFKey.Version > 10) // '字节解密'
                for (uint i = 0; i != size; ++i)
                    bytes[i] = __sub(bytes[i], xQFKey.F32Keys[(pos + i) % 32]);
        }

        let __getRemarksize=||
        {
            byte[] clen = new byte[4];
            __readBytes(clen, 4);
            return (uint)(clen[0] + (clen[1] << 8) + (clen[2] << 16) + (clen[3] << 24)) - xQFKey.KeyRMKSize;
        };

        // Encoding codec = Encoding.GetEncoding("gb2312"); // "gb2312"
        let data =[0u8;4];
        let frc = data[0];
        let trc = data[1];
        let tag = data[2];
        let __readDataAndGetRemark=||
        {
            __readBytes(data, 4);
            let RemarkSize = 0;
            frc = data[0];
            trc = data[1];
            tag = data[2];
            if xQFKey.Version <= 10
            {
                tag = (byte)((((tag & 0xF0) != 0) ? 0x80 : 0) | (((tag & 0x0F) != 0) ? 0x40 : 0));
                RemarkSize = __getRemarksize();
            }
            else
            {
                tag &= 0xE0;
                if ((tag & 0x20) != 0)
                    RemarkSize = __getRemarksize();
            }

            if (RemarkSize == 0)
                return null;

            // # 有注解
            byte[] rem = new byte[2048 * 2];
            __readBytes(rem, (int)RemarkSize);
            var remark = codec.GetString(rem).Replace('\0', ' ')
                .Replace("\r\n", "\n").Trim();
            return remark.Length > 0 ? remark : null;
        };

        // stream.Seek(1024, SeekOrigin.Begin);
        self.root_move.remark = __readDataAndGetRemark()?.Trim();

        if tag & 0x80 == 0 // 无左子树
            {return;}

        // 有左子树
        let beforeMoves = vec![];
        beforeMoves.push(self.root_move);
        let isOther = false;
        let beforeMove = self.root_move;
        // 当前棋子为根，且有后继棋子时，表明深度搜索已经回退到根，已经没有后续棋子了
        while !(beforeMove.before.upgrade().is_none() && beforeMove.after.borrow().len()>0)
        {
            let remark = __readDataAndGetRemark();
            //# 一步棋的起点和终点有简单的加密计算，读入时需要还原

            let fcolrow = __sub(frc, (0X18 + xQFKey.KeyXYf));
               let tcolrow = __sub(trc,(0X20 + xQFKey.KeyXYt));
            if fcolrow > 89 || tcolrow > 89
               { assert!(false,"fcolrow > 89 || tcolrow > 89 ? ");}

            let frow = 10 - 1 - fcolrow % 10;
            let fcol = fcolrow / 10;
               let trow = 10 - 1 - tcolrow % 10;
               let tcol = tcolrow / 10;//

            let coordPair =CoordPair::from_rowcol(frow as usize, fcol as usize, trow as usize, tcol as usize).unwrap();
            let hasNext = (tag & 0x80) != 0;
            let hasOther = (tag & 0x40) != 0;

            if beforeMove.CoordPair.Equals(coordPair)
            {
                Debug.WriteLine("Error: " + coordPair.ToString() + beforeMove.Remark);
            }
            else
            {
                if (isOther)
                    beforeMove = beforeMove.Before ?? beforeMove;

                beforeMove = beforeMove.AddAfter(coordPair, remark);
                if (hasNext && hasOther)
                    beforeMoves.Push(beforeMove);

                isOther = !hasNext;
                if (isOther && !hasOther && beforeMoves.Count > 0)
                {
                    beforeMove = beforeMoves.Pop(); // 最后时，将回退到根
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
