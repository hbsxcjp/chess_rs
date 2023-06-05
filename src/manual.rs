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
    source,
    title,
    game,
    date,
    site,
    black,
    rowCols,
    red,
    eccoSn,
    eccoName,
    win,
    opening,
    writer,
    author,
    atype,
    version,
    FEN,
    moveString,
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

    pub fn from_xqf(file_name: &str) -> Self {
        let manual = Manual::new();
        match std::fs::read(file_name) {
            Ok(byte_vec) => {
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
                // let &byte_vec = new byte[496];
                // stream.Read(&byte_vec, 0, 496);
                let signature = &byte_vec[0..2];
                let productid = &byte_vec[4..8];
                let headqizixy = &byte_vec[16..48];
                let playstepno = &byte_vec[48..50];
                let playnodes = &byte_vec[52..56];
                let ptreepos = &byte_vec[56..60];
                let reserved1 = &byte_vec[60..64];
                let headcodea_h = &byte_vec[64..80];
                let titlea = &byte_vec[80..144];
                let titleb = &byte_vec[144..208];
                let event = &byte_vec[208..272];
                let date = &byte_vec[272..288];
                let site = &byte_vec[288..304];
                let red = &byte_vec[304..320];
                let black = &byte_vec[320..336];
                let opening = &byte_vec[336..400];
                let redtime = &byte_vec[400..416];
                let blktime = &byte_vec[416..432];
                let reservedh = &byte_vec[432..464];
                let rmkwriter = &byte_vec[464..480];
                let author = &byte_vec[480..496]; //, Other[528]{};
                let version = byte_vec[2];
                let headkeymask = byte_vec[3];
                let headkeyora = byte_vec[8];
                let headkeyorb = byte_vec[9];
                let headkeyorc = byte_vec[10];
                let headkeyord = byte_vec[11];
                let headkeyssum = byte_vec[12];
                let headkeyxy = byte_vec[13];
                let headkeyxyf = byte_vec[14];
                let headkeyxyt = byte_vec[15];
                let headwhoplay = byte_vec[50];
                let headplayresult = byte_vec[51];

                if signature[0] != 0x58 || signature[1] != 0x51 {
                    assert!(false, "文件标记不符。");
                }
                if (headkeyssum as usize
                    + headkeyxy as usize
                    + headkeyxyf as usize
                    + headkeyxyt as usize)
                    % 256
                    != 0
                {
                    assert!(false, "检查密码校验和不对，不等于0。");
                }
                if version > 18 {
                    assert!(
                        false,
                        "这是一个高版本的XQF文件，您需要更高版本的XQStudio来读取这个文件。"
                    );
                }

                let keyxyf: u8;
                let keyxyt: u8;
                let keyrmksize: u32;
                let mut f32keys = [0; PIECENUM];

                let mut head_qizixy = [0; PIECENUM];
                for index in 0..PIECENUM {
                    head_qizixy[index] = headqizixy[index];
                }
                if version <= 10 {
                    // version <= 10 兼容1.0以前的版本
                    keyrmksize = 0;
                    keyxyf = 0;
                    keyxyt = 0;
                } else {
                    let KeyXY;
                    let calkey = |bkey, ckey| {
                        // % 256; // 保持为<256
                        ((((((bkey as usize * bkey as usize) * 3 + 9) * 3 + 8) * 2 + 1) * 3 + 8)
                            * ckey as usize) as u8
                    };

                    KeyXY = calkey(headkeyxy, headkeyxy);
                    keyxyf = calkey(headkeyxyf, KeyXY);
                    keyxyt = calkey(headkeyxyt, keyxyf);
                    keyrmksize = ((headkeyssum as u32 * 256 + headkeyxy as u32) % 32000) + 767; // % 65536
                    if version >= 12 {
                        // 棋子位置循环移动
                        let Qixy = headqizixy.clone();
                        for index in 0..PIECENUM {
                            head_qizixy[(index + KeyXY as usize + 1) % PIECENUM] = Qixy[index];
                        }
                    }
                    for index in 0..PIECENUM {
                        head_qizixy[index] -= KeyXY;
                    } // 保持为8位无符号整数，<256
                }

                let keybytes = [
                    (headkeyssum & headkeymask) | headkeyora,
                    (headkeyxy & headkeymask) | headkeyorb,
                    (headkeyxyf & headkeymask) | headkeyorc,
                    (headkeyxyt & headkeymask) | headkeyord,
                ];
                let mut index = 0;
                for ch in "[(C) Copyright Mr. Dong Shiwei.]".bytes() {
                    f32keys[index] = ch & keybytes[index % 4];
                    index += 1;
                } // ord(c)

                // 取得棋子字符串
                let mut piece_chars = vec![b'_'; SEATCOUNT];
                let mut index = 0;
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

                // Info = new();
                // System.Text.Encoding.RegisterProvider(System.Text.CodePagesEncodingProvider.Instance);
                // Encoding.RegisterProvider(CodePagesEncodingProvider.Instance);
                // Encoding codec = Encoding.GetEncoding("gb2312"); // "gb2312"
                let result = ["未知", "红胜", "黑胜", "和棋"];
                let typestr = ["全局", "开局", "中局", "残局"];
                let GetInfoString = |byte_gbk| {
                    GBK.decode(byte_gbk, DecoderTrap::Strict)
                        .unwrap()
                        .replace('\0', " ")
                        .trim();
                };

                for (key, value) in [
                    (InfoKey::FEN, fen + " r - - 0 1"), // 可能存在不是红棋先走的情况？
                    (InfoKey::version, version.to_string()),
                    // (InfoKey::win, result[headPlayResult].Trim()),
                    // (InfoKey::atype, typestr[headCodeA_H[0]].Trim()),
                    // (InfoKey::title, GetInfoString(TitleA)),
                    // (InfoKey::game, GetInfoString(Event)),
                    // (InfoKey::date, GetInfoString(Date)),
                    // (InfoKey::site, GetInfoString(Site)),
                    // (InfoKey::red, GetInfoString(Red)),
                    // (InfoKey::black, GetInfoString(Black)),
                    // (InfoKey::opening, GetInfoString(Opening)),
                    // (InfoKey::writer, GetInfoString(RMKWriter)),
                    // (InfoKey::author, GetInfoString(Author)),
                ] {
                    // SetInfoValue(fieldValue.field, fieldValue.value);
                }

                // ManualMove = new(fen, stream, (Version, KeyXYf, KeyXYt, KeyRMKSize, F32Keys));
            }
            _ => {}
        };

        manual
    }

    pub fn to_string(&self) -> String {
        // format!(
        //     "{:?}.{:?} {} {}",
        //     self.before.upgrade().unwrap(),
        //     self.id,
        //     self.coordpair.to_string(),
        //     self.remark.borrow()
        // )

        String::new()
    }
}