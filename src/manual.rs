#![allow(dead_code)]

use crate::coord::{self, COLCOUNT, ROWCOUNT, SEATCOUNT};
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
// use serde_derive::{Deserialize, Serialize};
// use serde_json;
// use regex::Error;
// use std::fs::File;
// use std::io::prelude::*;
// use std::io::BufReader;
// use serde::de::value;
use crate::board;
use crate::common;
use crate::evaluation;
use crate::manual_move;
use std::borrow::Borrow;
use std::collections::BTreeMap;
// use std::rc::Rc;
use num_enum::TryFromPrimitive;

pub type ManualInfo = BTreeMap<String, String>;

#[derive(Debug, TryFromPrimitive)]
#[repr(usize)]
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

pub fn get_fen(info: &ManualInfo) -> &str {
    if let Some(value) = info.get(&InfoKey::FEN.to_string()) {
        if !value.is_empty() {
            return value.split_once(" ").unwrap().0;
        }
    }

    board::FEN
}

#[derive(Debug)]
pub struct Manual {
    info: ManualInfo,
    pub manual_move: manual_move::ManualMove,
}

impl PartialEq for Manual {
    fn eq(&self, other: &Self) -> bool {
        self.manual_move == other.manual_move
    }
}

impl Manual {
    pub fn new() -> Self {
        Manual {
            info: ManualInfo::new(),
            manual_move: manual_move::ManualMove::new(),
        }
    }

    pub fn from(file_name: &str) -> common::Result<Self> {
        let record_type = coord::RecordType::get_record_type(file_name)?;
        match record_type {
            coord::RecordType::Xqf => Self::from_xqf(file_name),
            coord::RecordType::Bin => Self::from_bin(file_name),
            _ => Self::from_string(file_name, record_type),
        }
    }

    pub fn from_info(info: ManualInfo) -> common::Result<Self> {
        let fen = get_fen(&info);
        let manual_move_str = info.get(&InfoKey::MoveString.to_string()).unwrap();
        let manual_move = match manual_move_str.is_empty() {
            true => {
                let rowcols_str = info.get(&InfoKey::RowCols.to_string()).unwrap();
                manual_move::ManualMove::from_rowcols(fen, rowcols_str)
            }
            false => {
                // println!("manual_move_str:{manual_move_str}\n");
                // println!("fen:{fen}\n");
                manual_move::ManualMove::from_string(fen, manual_move_str, coord::RecordType::PgnZh)
            }
        }?;

        Ok(Manual { info, manual_move })
    }

    pub fn get_info(&self) -> ManualInfo {
        self.info.clone()
    }

    // pub fn set_info(&mut self, key: String, value: String) {
    //     if let Some(ref_value) = self.info.get_mut(&key) {
    //         *ref_value = value;
    //     } else {
    //         self.info.insert(key, value);
    //     }
    // }

    pub fn write(&self, file_name: &str) -> Result<(), std::io::ErrorKind> {
        let record_type =
            coord::RecordType::get_record_type(file_name).map_err(|_| std::io::ErrorKind::Other)?;
        match record_type {
            coord::RecordType::Xqf => Err(std::io::ErrorKind::Other),
            coord::RecordType::Bin => {
                std::fs::write(&file_name, self.get_bytes()).map_err(|_| std::io::ErrorKind::Other)
            }
            _ => std::fs::write(&file_name, self.to_string(record_type))
                .map_err(|_| std::io::ErrorKind::Other),
        }
    }

    fn from_xqf(file_name: &str) -> common::Result<Self> {
        let mut info = ManualInfo::new();
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
                info.insert(key.to_string(), value);
            }

            manual_move = manual_move::ManualMove::from_xqf(
                &fen, &input, version, keyxyf, keyxyt, keyrmksize, &f32keys,
            );
        }

        Ok(Manual { info, manual_move })
    }

    pub fn from_bin(file_name: &str) -> common::Result<Self> {
        let mut info = ManualInfo::new();
        let mut manual_move = manual_move::ManualMove::new();
        if let Ok(input) = std::fs::read(file_name) {
            let mut input = input.borrow();
            let info_len = common::read_be_u32(&mut input);
            for _ in 0..info_len {
                let key = common::read_string(&mut input);
                let value = common::read_string(&mut input);

                // println!("key_value: {key} = {value}");
                info.insert(key, value);
            }

            let fen = get_fen(&info);
            manual_move = manual_move::ManualMove::from_bin(&fen, &mut input);
        }

        Ok(Manual { info, manual_move })
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        common::write_be_u32(&mut result, self.info.len() as u32);
        for (key, value) in &self.info {
            common::write_string(&mut result, key);
            common::write_string(&mut result, value);
        }
        result.append(&mut self.manual_move.get_bytes());

        result
    }

    pub fn from_string(file_name: &str, record_type: coord::RecordType) -> common::Result<Self> {
        let manual_string =
            std::fs::read_to_string(&file_name).map_err(|_| common::ParseError::StringParse)?;
        let (info_str, manual_move_str) = manual_string
            .split_once("\n\n")
            .ok_or(common::ParseError::StringParse)?;

        let mut info = ManualInfo::new();
        let info_re = regex::Regex::new(r"\[(\S+): ([\s\S]*?)\]").unwrap();
        for caps in info_re.captures_iter(info_str) {
            let key = caps.at(1).unwrap().to_string();
            let value = caps.at(2).unwrap().to_string();

            info.insert(key, value);
        }

        let fen = get_fen(&info);
        // if record_type == coord::RecordType::PgnZh {
        //     println!("info:{:?}\nfen:{}", info, fen);
        // }
        let manual_move = manual_move::ManualMove::from_string(fen, manual_move_str, record_type)?;

        Ok(Manual { info, manual_move })
    }

    pub fn get_zorbist_evaluation(&self) -> evaluation::ZorbistEvaluation {
        self.manual_move.get_zorbist_evaluation()
    }

    pub fn to_rowcols(&self) -> String {
        self.manual_move.to_rowcols()
    }

    pub fn get_manualmove_string(&self, record_type: coord::RecordType) -> String {
        self.manual_move.to_string(record_type)
    }

    pub fn to_string(&self, record_type: coord::RecordType) -> String {
        let mut info_str = String::new();
        for (key, value) in &self.info {
            info_str.push_str(&format!("[{key}: {value}]\n"));
        }

        format!("{}\n{}", info_str, self.get_manualmove_string(record_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual() {
        let manual = Manual::new();
        assert_eq!("\n\n", manual.to_string(coord::RecordType::Txt));

        fn get_file_path(file_name: &str, record_type: coord::RecordType) -> String {
            format!("tests/output/{}.{}", file_name, record_type.ext_name())
        }

        let filename_manuals = common::get_filename_manuals();
        for (file_name, manual_string, manual) in filename_manuals {
            assert_eq!(manual_string, manual.to_string(coord::RecordType::Txt));

            // 输出内容以备查看
            for record_type in [
                coord::RecordType::Bin,
                coord::RecordType::Txt,
                coord::RecordType::PgnIccs,
                coord::RecordType::PgnRc,
                coord::RecordType::PgnZh,
            ] {
                let file_path = get_file_path(file_name, record_type);
                if std::fs::File::open(&file_path).is_err() {
                    let _ = manual.write(&file_path);
                }

                let manual = Manual::from(&file_path).unwrap();
                assert_eq!(
                    manual_string,
                    manual.to_string(coord::RecordType::Txt),
                    "file_path: {file_path}"
                );
            }
        }
    }
}
