#![allow(dead_code)]

use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
// use crate::amove;
use crate::bit_constant::COLCOUNT;
use crate::bit_constant::ROWCOUNT;
use crate::bit_constant::SEATCOUNT;
use crate::manual_move;
// use crate::bit_constant;
use crate::board;
use std::collections::HashMap;
// use std::rc::Rc;

#[derive(Debug)]
pub enum InfoKey {
    Source,
    Title,
    Game,
    Date,
    Site,
    Black,
    RowCols,
    Red,
    EccoSn,
    EccoName,
    Win,
    Opening,
    Writer,
    Author,
    Atype,
    Version,
    FEN,
    MoveString,
}

impl InfoKey {
    pub fn to_string(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug)]
pub struct Manual {
    pub info: HashMap<String, String>,

    pub manual_move: manual_move::ManualMove,
}

impl Manual {
    pub fn new() -> Self {
        Manual {
            info: HashMap::new(),
            manual_move: manual_move::ManualMove::new(board::FEN),
        }
    }

    fn from_xqf(file_name: &str) -> Self {
        let mut manual = Manual::new();
        if let Ok(byte_vec) = std::fs::read(file_name) {
            //文件标记'XQ'=$5158/版本/加密掩码/ProductId[4], 产品(厂商的产品号)
            // 棋谱评论员/文件的作者
            // 32个棋子的原始位置
            // 加密的钥匙和/棋子布局位置钥匙/棋谱起点钥匙/棋谱终点钥匙
            // 用单字节坐标表示, 将字节变为十进制, 十位数为X(0-8)个位数为Y(0-9),
            // 棋盘的左下角为原点(0, 0). 32个棋子的位置从1到32依次为:
            // 红: 车马相士帅士相马车炮炮兵兵兵兵兵 (位置从右到左, 从下到上)
            // 黑: 车马象士将士象马车炮炮卒卒卒卒卒 (位置从右到左,
            // 该谁下 0-红先, 1-黑先/最终结果 0-未知, 1-红胜 2-黑胜, 3-和棋
            // 从下到上)PlayStepNo[2],
            // 对局类型(开,中,残等)
            const PIECENUM: usize = 32;
            let signature = &byte_vec[0..2];
            // let productid = &byte_vec[4..8];
            let headqizixy = &byte_vec[16..48];
            // let playstepno = &byte_vec[48..50];
            // let playnodes = &byte_vec[52..56];
            // let ptreepos = &byte_vec[56..60];
            // let reserved1 = &byte_vec[60..64];
            let headcodea_h = &byte_vec[64..80];
            let titlea = &byte_vec[80..144];
            // let titleb = &byte_vec[144..208];
            let event = &byte_vec[208..272];
            let date = &byte_vec[272..288];
            let site = &byte_vec[288..304];
            let red = &byte_vec[304..320];
            let black = &byte_vec[320..336];
            let opening = &byte_vec[336..400];
            // let redtime = &byte_vec[400..416];
            // let blktime = &byte_vec[416..432];
            // let reservedh = &byte_vec[432..464];
            let rmkwriter = &byte_vec[464..480];
            let author = &byte_vec[480..496]; //, Other[528]{};
            let version = byte_vec[2];
            let headkeymask = byte_vec[3];
            let headkeyora = byte_vec[8];
            let headkeyorb = byte_vec[9];
            let headkeyorc = byte_vec[10];
            let headkeyord = byte_vec[11];
            let headkeyssum = byte_vec[12] as usize;
            let headkeyxy = byte_vec[13] as usize;
            let headkeyxyf = byte_vec[14] as usize;
            let headkeyxyt = byte_vec[15] as usize;
            // let headwhoplay = byte_vec[50];
            let headplayresult = byte_vec[51] as usize;

            if signature[0] != 0x58 || signature[1] != 0x51 {
                assert!(false, "文件标记不符。");
            }
            if (headkeyssum + headkeyxy + headkeyxyf + headkeyxyt) % 256 != 0 {
                assert!(false, "检查密码校验和不对，不等于0。");
            }
            if version > 18 {
                assert!(
                    false,
                    "这是一个高版本的XQF文件，您需要更高版本的XQStudio来读取这个文件。"
                );
            }

            let keyxyf: usize;
            let keyxyt: usize;
            let keyrmksize: usize;
            let mut f32keys = [0; PIECENUM];

            let mut head_qizixy = [0; PIECENUM];
            for index in 0..PIECENUM {
                head_qizixy[index] = headqizixy[index] as usize;
            }
            // version <= 10 兼容1.0以前的版本
            if version <= 10 {
                keyrmksize = 0;
                keyxyf = 0;
                keyxyt = 0;
            } else {
                let keyxy;
                let calkey = |bkey, ckey| {
                    // % 256; // 保持为<256
                    ((((((bkey * bkey) * 3 + 9) * 3 + 8) * 2 + 1) * 3 + 8) * ckey) as u8 as usize
                };

                keyxy = calkey(headkeyxy, headkeyxy);
                keyxyf = calkey(headkeyxyf, keyxy);
                keyxyt = calkey(headkeyxyt, keyxyf);
                // % 65536
                keyrmksize = ((headkeyssum * 256 + headkeyxy) % 32000) + 767;
                // 棋子位置循环移动
                if version >= 12 {
                    let qixy = headqizixy.clone();
                    for index in 0..PIECENUM {
                        head_qizixy[(index + keyxy + 1) % PIECENUM] = qixy[index] as usize;
                    }
                }
                for qizixy in &mut head_qizixy {
                    // 保持为8位无符号整数，<256
                    *qizixy = (*qizixy - keyxy) as u8 as usize;
                }
            }

            let keybytes = [
                (headkeyssum as u8 & headkeymask) | headkeyora,
                (headkeyxy as u8 & headkeymask) | headkeyorb,
                (headkeyxyf as u8 & headkeymask) | headkeyorc,
                (headkeyxyt as u8 & headkeymask) | headkeyord,
            ];
            let mut index = 0;
            for ch in "[(C) Copyright Mr. Dong Shiwei.]".bytes() {
                f32keys[index] = ch & keybytes[index % 4];
                index += 1;
            } // ord(c)

            // 取得棋子字符串
            let mut piece_chars = vec![b'_'; SEATCOUNT];
            index = 0;
            // QiziXY设定的棋子顺序
            for ch in "RNBAKABNRCCPPPPPrnbakabnrccppppp".bytes() {
                let xy = head_qizixy[index] as usize;
                if xy < SEATCOUNT {
                    // 用单字节坐标表示, 将字节变为十进制,
                    // 十位数为X(0-8),个位数为Y(0-9),棋盘的左下角为原点(0, 0)
                    piece_chars[(ROWCOUNT - xy % ROWCOUNT) * COLCOUNT + xy / ROWCOUNT] = ch;
                }
                index += 1;
            }

            let fen = board::piece_chars_to_fen(&String::from_utf8(piece_chars).unwrap());
            let result = ["未知", "红胜", "黑胜", "和棋"];
            let typestr = ["全局", "开局", "中局", "残局"];
            let bytes_to_string = |bytes| {
                GBK.decode(bytes, DecoderTrap::Ignore).unwrap()
                // .replace('\0', " ")
                // .trim()
            };

            for (key, value) in [
                (InfoKey::FEN, format!("{fen} + r - - 0 1")), // 可能存在不是红棋先走的情况？
                (InfoKey::Version, version.to_string()),
                (InfoKey::Win, String::from(result[headplayresult as usize])),
                (
                    InfoKey::Atype,
                    String::from(typestr[headcodea_h[0] as usize]),
                ),
                (InfoKey::Title, bytes_to_string(titlea)),
                (InfoKey::Game, bytes_to_string(event)),
                (InfoKey::Date, bytes_to_string(date)),
                (InfoKey::Site, bytes_to_string(site)),
                (InfoKey::Red, bytes_to_string(red)),
                (InfoKey::Black, bytes_to_string(black)),
                (InfoKey::Opening, bytes_to_string(opening)),
                (InfoKey::Writer, bytes_to_string(rmkwriter)),
                (InfoKey::Author, bytes_to_string(author)),
            ] {
                manual.set_infokey_value(key.to_string(), value);
            }

            manual.manual_move.set_from(
                &fen, &byte_vec, version, keyxyf, keyxyt, keyrmksize, &f32keys,
            );
        }

        manual
    }

    fn set_infokey_value(&mut self, key: String, value: String) {
        self.info.insert(key, value);
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (key, value) in &self.info {
            result.push_str(&format!("[{key} {value}]\n"));
        }
        result.push_str(&self.manual_move.to_string());

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual() {
        let manual = Manual::new();

        assert_eq!("[(0,0)->(0,0)] \n", manual.to_string());
    }
}
