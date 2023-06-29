#![allow(dead_code)]

use crate::coord;
use crate::coord::{COLCOUNT, ROWCOUNT, SEATCOUNT};
use crate::manual_move;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
// use std::fs::File;
// use std::io::prelude::*;
// use std::io::BufReader;
// use serde::de::value;
// use crate::bit_constant;
use crate::board;
use crate::utility;
use std::borrow::Borrow;
use std::collections::BTreeMap;
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

#[derive(Debug)]
pub struct Manual {
    pub info: BTreeMap<String, String>,
    pub manual_move: manual_move::ManualMove,
}

impl Manual {
    pub fn new() -> Self {
        Manual {
            info: BTreeMap::new(),
            manual_move: manual_move::ManualMove::new(),
        }
    }

    pub fn from(file_name: &str) -> Self {
        let record_type = coord::RecordType::get_record_type(file_name);
        match record_type {
            coord::RecordType::Xqf => Self::from_xqf(file_name),
            coord::RecordType::Bin => Self::from_bin(file_name),
            _ => Self::from_string(file_name, record_type),
        }
    }

    pub fn write(&self, file_name: &str) {
        let record_type = coord::RecordType::get_record_type(file_name);
        match record_type {
            coord::RecordType::Xqf => (),
            coord::RecordType::Bin => {
                std::fs::write(&file_name, self.get_bytes()).expect("Write Err.")
            }
            _ => std::fs::write(&file_name, self.to_string(record_type)).expect("Write Err."),
        };
    }

    fn from_xqf(file_name: &str) -> Self {
        let mut info = BTreeMap::new();
        let mut manual_move = manual_move::ManualMove::new();
        if let Ok(input) = std::fs::read(file_name) {
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
            let signature = &input[0..2];
            // let productid = &byte_vec[4..8];
            let headqizixy = &input[16..48];
            // let playstepno = &byte_vec[48..50];
            // let playnodes = &byte_vec[52..56];
            // let ptreepos = &byte_vec[56..60];
            // let reserved1 = &byte_vec[60..64];
            let headcodea_h = &input[64..80];
            let titlea = &input[80..144];
            // let titleb = &byte_vec[144..208];
            let event = &input[208..272];
            let date = &input[272..288];
            let site = &input[288..304];
            let red = &input[304..320];
            let black = &input[320..336];
            let opening = &input[336..400];
            // let redtime = &byte_vec[400..416];
            // let blktime = &byte_vec[416..432];
            // let reservedh = &byte_vec[432..464];
            let rmkwriter = &input[464..480];
            let author = &input[480..496]; //, Other[528]{};
            let version = input[2];
            let headkeymask = input[3];
            let headkeyora = input[8];
            let headkeyorb = input[9];
            let headkeyorc = input[10];
            let headkeyord = input[11];
            let headkeyssum = input[12] as usize;
            let headkeyxy = input[13] as usize;
            let headkeyxyf = input[14] as usize;
            let headkeyxyt = input[15] as usize;
            // let headwhoplay = byte_vec[50];
            let headplayresult = input[51] as usize;

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

            let mut head_qizixy = headqizixy.to_vec();
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
                    for (index, qizixy) in headqizixy.iter().enumerate() {
                        head_qizixy[(index + keyxy + 1) % PIECENUM] = *qizixy;
                    }
                }
                for qizixy in &mut head_qizixy {
                    // 保持为8位无符号整数，<256
                    *qizixy = (*qizixy as isize - keyxy as isize) as u8;
                }
            }

            let keybytes = [
                (headkeyssum as u8 & headkeymask) | headkeyora,
                (headkeyxy as u8 & headkeymask) | headkeyorb,
                (headkeyxyf as u8 & headkeymask) | headkeyorc,
                (headkeyxyt as u8 & headkeymask) | headkeyord,
            ];
            for (index, ch) in "[(C) Copyright Mr. Dong Shiwei.]".bytes().enumerate() {
                f32keys[index] = ch & keybytes[index % 4];
            } // ord(c)

            // 取得棋子字符串
            let mut piece_chars = vec![b'_'; SEATCOUNT];
            // QiziXY设定的棋子顺序
            for (index, ch) in "RNBAKABNRCCPPPPPrnbakabnrccppppp".bytes().enumerate() {
                let xy = head_qizixy[index] as usize;
                if xy < SEATCOUNT {
                    // 用单字节坐标表示, 将字节变为十进制,
                    // 十位数为X(0-8),个位数为Y(0-9),棋盘的左下角为原点(0, 0)
                    piece_chars[(ROWCOUNT - 1 - xy % ROWCOUNT) * COLCOUNT + xy / ROWCOUNT] = ch;
                }
            }

            let fen = board::piece_chars_to_fen(&String::from_utf8(piece_chars).unwrap());
            let result = ["未知", "红胜", "黑胜", "和棋"];
            let typestr = ["全局", "开局", "中局", "残局"];
            let bytes_to_string = |bytes| {
                GBK.decode(bytes, DecoderTrap::Ignore)
                    .unwrap()
                    .replace('\0', "")
                    .trim()
                    .into()
            };

            for (key, value) in [
                (InfoKey::FEN, format!("{fen} r - - 0 1")), // 可能存在不是红棋先走的情况？
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
                info.insert(format!("{:?}", key), value);
            }

            manual_move = manual_move::ManualMove::from_xqf(
                &fen, &input, version, keyxyf, keyxyt, keyrmksize, &f32keys,
            );
        }

        Manual { info, manual_move }
    }

    fn set_infokey_value(&mut self, key: String, value: String) {
        self.info.insert(key, value);
    }

    fn get_fen(info: &BTreeMap<String, String>) -> &str {
        match info.get(&format!("{:?}", InfoKey::FEN)) {
            Some(value) => value.split_once(" ").unwrap().0,
            None => board::FEN,
        }
    }

    fn fen(&self) -> &str {
        Self::get_fen(&self.info)
    }

    pub fn from_bin(file_name: &str) -> Self {
        let mut info = BTreeMap::new();
        let mut manual_move = manual_move::ManualMove::new();
        if let Ok(input) = std::fs::read(file_name) {
            let mut input = input.borrow();
            let info_len = utility::read_be_u32(&mut input);
            for _ in 0..info_len {
                let key = utility::read_string(&mut input);
                let value = utility::read_string(&mut input);

                // println!("key_value: {key} = {value}");
                info.insert(key, value);
            }
            let fen = info
                .get(&format!("{:?}", InfoKey::FEN))
                .unwrap()
                .split(' ')
                .collect::<Vec<&str>>()[0];

            manual_move = manual_move::ManualMove::from_bin(fen, &mut input);
        }

        Manual { info, manual_move }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        utility::write_be_u32(&mut result, self.info.len() as u32);
        for (key, value) in &self.info {
            utility::write_string(&mut result, key);
            utility::write_string(&mut result, value);
        }
        result.append(&mut self.manual_move.get_bytes());

        result
    }

    pub fn from_string(file_name: &str, record_type: coord::RecordType) -> Self {
        let manual_string = std::fs::read_to_string(&file_name).unwrap();
        let (info_str, manual_move_str) = manual_string.split_once("\n\n").unwrap();

        let mut info = BTreeMap::new();
        let info_re = regex::Regex::new(r"\[(\S+): ([\s\S]*)\]").unwrap();
        for caps in info_re.captures_iter(info_str) {
            let key = caps.at(1).unwrap().to_string();
            let value = caps.at(2).unwrap().to_string();

            info.insert(key, value);
        }
        // println!("{:?}", info);

        let fen = Self::get_fen(&info);
        let manual_move = manual_move::ManualMove::from_string(fen, manual_move_str, record_type);

        Manual { info, manual_move }
    }

    pub fn to_string(&self, record_type: coord::RecordType) -> String {
        let mut remark = String::new();
        for (key, value) in &self.info {
            remark.push_str(&format!("[{key}: {value}]\n"));
        }

        format!("{}\n{}", remark, self.manual_move.to_string(record_type))
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    #[test]
    fn test_manual() {}
}
