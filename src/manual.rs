#![allow(dead_code)]

use crate::common;
use crate::coord::{self, COLCOUNT, ROWCOUNT, SEATCOUNT};
use crate::evaluation;
use crate::manual_move;
use crate::models::ManualInfo;
use crate::{board, models};
use diesel::sqlite::SqliteConnection;
use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
use std::borrow::{Borrow, BorrowMut};
use std::path::{Path, PathBuf};
// use std::cell::RefCell;
// use diesel::prelude::*;
// use diesel::result::Error;

#[derive(Debug)]
pub struct Manual {
    info: models::ManualInfo,
    manual_move: manual_move::ManualMove,
}

impl PartialEq for Manual {
    fn eq(&self, other: &Self) -> bool {
        self.manual_move == other.manual_move
    }
}

impl Manual {
    pub fn new() -> Self {
        Manual::from(models::ManualInfo::new(), manual_move::ManualMove::new())
    }

    fn from(info: models::ManualInfo, manual_move: manual_move::ManualMove) -> Self {
        Manual { info, manual_move }
    }

    pub fn from_path(path: &Path) -> common::Result<Self> {
        if let Some(record_type) = coord::RecordType::get_record_type(path) {
            match record_type {
                coord::RecordType::Xqf => Self::from_xqf(path),
                coord::RecordType::Bin => Self::from_bin(path),
                _ => Self::from_string(path, record_type),
            }
        } else {
            Err(common::ParseError::StringParse)
        }
    }

    pub fn from_info(info: models::ManualInfo) -> common::Result<Self> {
        let fen = info.get_fen();
        let manual_move = if let Some(manual_move_str) = &info.movestring {
            manual_move::ManualMove::from_string(fen, &manual_move_str, coord::RecordType::Txt)?
        } else if let Some(rowcols_str) = info.rowcols.as_ref() {
            manual_move::ManualMove::from_rowcols(fen, rowcols_str)?
        } else {
            return Err(common::ParseError::StringParse);
        };

        Ok(Manual::from(info, manual_move))
    }

    pub fn write(&self, path: &Path) -> Result<(), std::io::ErrorKind> {
        if let Some(record_type) = coord::RecordType::get_record_type(path) {
            match record_type {
                coord::RecordType::Xqf => Err(std::io::ErrorKind::Other),
                coord::RecordType::Bin => {
                    std::fs::write(&path, self.get_bytes()).map_err(|_| std::io::ErrorKind::Other)
                }
                _ => std::fs::write(&path, self.to_string_type(record_type))
                    .map_err(|_| std::io::ErrorKind::Other),
            }
        } else {
            Err(std::io::ErrorKind::Other)
        }
    }

    fn from_xqf(path: &Path) -> common::Result<Self> {
        let mut info = models::ManualInfo::new();
        let mut manual_move = manual_move::ManualMove::new();
        if let Ok(input) = std::fs::read(path) {
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

            info.fen = Some(format!("{fen} r - - 0 1")); // 可能存在不是红棋先走的情况？
            info.version = Some(version.to_string());
            info.win = Some(String::from(result[headplayresult as usize]));
            info.atype = Some(String::from(typestr[headcodea_h[0] as usize]));
            info.title = bytes_to_string(titlea);
            info.game = bytes_to_string(event);
            info.date = Some(bytes_to_string(date));
            info.site = Some(bytes_to_string(site));
            info.red = Some(bytes_to_string(red));
            info.black = Some(bytes_to_string(black));
            info.opening = Some(bytes_to_string(opening));
            info.writer = Some(bytes_to_string(rmkwriter));
            info.author = Some(bytes_to_string(author));

            manual_move = manual_move::ManualMove::from_xqf(
                &fen, &input, version, keyxyf, keyxyt, keyrmksize, &f32keys,
            );
        }

        Ok(Manual::from(info, manual_move))
    }

    fn from_bin(path: &Path) -> common::Result<Self> {
        if let Ok(input) = std::fs::read(path) {
            let mut input = input.borrow();
            let mut key_values = vec![];
            let info_len = common::read_be_u32(&mut input);
            for _ in 0..info_len {
                let key = common::read_string(&mut input);
                let value = common::read_string(&mut input);

                key_values.push((key, value));
                // println!("key_value: {key} = {value}");
                // info_old.insert(key, value);
            }

            // let fen = get_fen_old(&info_old);
            let info = models::ManualInfo::from(key_values);
            let fen = info.get_fen();
            let manual_move = manual_move::ManualMove::from_bin(fen, &mut input);
            Ok(Manual::from(info, manual_move))
        } else {
            Err(common::ParseError::StringParse)
        }
    }

    pub fn get_bytes(&self) -> Vec<u8> {
        let mut result = Vec::new();
        let key_values = self.info.borrow().get_key_values();
        common::write_be_u32(&mut result, key_values.len() as u32);
        for (key, value) in key_values {
            common::write_string(&mut result, key);
            common::write_string(&mut result, value);
        }
        result.append(&mut self.manual_move.get_bytes());

        result
    }

    pub fn get_zorbist(&self) -> evaluation::Zorbist {
        self.manual_move.get_zorbist()
    }

    fn from_string(path: &Path, record_type: coord::RecordType) -> common::Result<Self> {
        let manual_string =
            std::fs::read_to_string(&path).map_err(|_| common::ParseError::StringParse)?;
        let (info_str, manual_move_str) = manual_string
            .split_once("\n\n")
            .ok_or(common::ParseError::StringParse)?;

        let mut key_values = vec![];
        let info_re = regex::Regex::new(r"\[(\S+): ([\s\S]*?)\]").unwrap();
        for caps in info_re.captures_iter(info_str) {
            let key = caps.at(1).unwrap().to_string();
            let value = caps.at(2).unwrap().to_string();
            key_values.push((key, value));
        }

        let info = models::ManualInfo::from(key_values);
        let fen = info.get_fen();
        // if record_type == coord::RecordType::PgnZh {
        //     println!("info:{:?}\nfen:{}", info, fen);
        // }
        let manual_move = manual_move::ManualMove::from_string(fen, manual_move_str, record_type)?;

        Ok(Manual::from(info, manual_move))
    }

    fn to_string_type(&self, record_type: coord::RecordType) -> String {
        let mut info_str = String::new();
        for (key, value) in self.info.borrow().get_key_values() {
            info_str.push_str(&format!("[{key}: {value}]\n"));
        }

        format!("{}\n{}", info_str, self.manual_move.to_string(record_type))
    }

    pub fn to_string(&self) -> String {
        self.to_string_type(coord::RecordType::Txt)
    }
}

pub fn save_manuals_to_db(
    filename_manuals: &Vec<(&str, &str, Manual)>,
    conn: &mut SqliteConnection,
) -> Result<usize, diesel::result::Error> {
    let mut infos = vec![];
    for (file_name, _, manual) in filename_manuals {
        let mut info = manual.info.get_copy();
        info.set_source_moves(
            file_name,
            &manual.manual_move.get_rowcols(),
            &manual.manual_move.to_string(coord::RecordType::Txt),
        );
        infos.push(info);
    }

    ManualInfo::save_db(&infos, conn)
}

pub fn read_manuals_from_db(
    conn: &mut SqliteConnection,
    title_part: &str,
) -> Result<Vec<Manual>, diesel::result::Error> {
    let mut result = vec![];
    let infos = models::ManualInfo::from_db(conn, title_part)?;
    for info in infos {
        if let Ok(manual) = Manual::from_info(info) {
            result.push(manual);
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manual_from_file() {
        let manual = Manual::new();
        assert_eq!("[title: 未命名]\n[game: 人机对战]\n[fen: rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR r - - 0 1]\n\n\n", 
            manual.to_string());

        let filename_manuals = common::get_filename_manuals();
        for (file_name, manual_string, manual) in filename_manuals {
            println!("file_name: {}", file_name);
            assert_eq!(manual_string, manual.to_string());

            // 输出内容以备查看
            for record_type in [
                coord::RecordType::Bin,
                coord::RecordType::Txt,
                coord::RecordType::PgnIccs,
                coord::RecordType::PgnRc,
                coord::RecordType::PgnZh,
            ] {
                let mut path = PathBuf::from("tests/output");
                path.push(file_name);
                path.set_extension(record_type.ext_name());
                // let path =Path::new(&format!("tests/output/{}.{}", file_name, record_type.ext_name()));
                if std::fs::File::open(&path).is_err() {
                    let _ = manual.write(&path);
                }

                let manual = Manual::from_path(&path).unwrap();
                assert_eq!(manual_string, manual.to_string(), "file_path: {path:?}");
            }
        }
    }

    #[test]
    #[ignore = "从样板文件提取manual后存入数据库。"]
    fn test_manual_from_file_to_db() {
        let conn = &mut models::get_conn();
        let filename_manuals = common::get_filename_manuals();
        let save_count = save_manuals_to_db(&filename_manuals, conn).unwrap();

        println!("manual save: {}", save_count);
    }

    #[test]
    // #[ignore = "从目录下xqf文件提取manual后存入数据库。"]
    fn test_manual_from_dir_to_db() {
        use std::fs;
        use std::io;
        let mut entries = fs::read_dir(".")
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap();
        // 不保证 `read_dir` 返回条目的顺序。
        // 如果需要可重复的排序，则应对条目进行显式排序。
        entries.sort();
        // 现在，条目已经按照路径进行了排序。
        // println!("{:?}", entries);

        let conn = &mut models::get_conn();
        let filename_manuals = vec![];
        let save_count = save_manuals_to_db(&filename_manuals, conn).unwrap();

        println!("manual save: {}", save_count);
    }

    #[test]
    #[ignore = "从数据库提取符合条件的manual后检验。"]
    fn test_manual_from_db_some() {
        let conn = &mut models::get_conn();
        let filename_manuals = common::get_filename_manuals();
        let mut read_count = 0;
        let manuals = read_manuals_from_db(conn, "%01%");
        if let Ok(mut manuals) = manuals {
            read_count = manuals.len();
            if let Some(manual) = manuals.get_mut(0) {
                manual.info.cut_source_moves();
                assert_eq!(filename_manuals[0].1, manual.to_string());
            }
        }
        println!("manual read some: {}", read_count);
    }

    #[test]
    #[ignore = "从数据库提取全部manual。"]
    fn test_manual_from_db_all() {
        let conn = &mut models::get_conn();
        let mut read_count = 0;
        if let Ok(manuals) = read_manuals_from_db(conn, "%") {
            read_count = manuals.len();
        }

        println!("manual read all: {}", read_count);
    }
}
