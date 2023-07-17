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

#[derive(Debug)]
pub struct Manual {
    pub info: ManualInfo,
    manual_move: manual_move::ManualMove,
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
                info.insert(format!("{:?}", key), value);
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
            let fen = info
                .get(&format!("{:?}", InfoKey::FEN))
                .unwrap()
                .split(' ')
                .collect::<Vec<&str>>()[0];

            manual_move = manual_move::ManualMove::from_bin(fen, &mut input);
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
        let info_re = regex::Regex::new(r"\[(\S+): ([\s\S]*)\]").unwrap();
        for caps in info_re.captures_iter(info_str) {
            let key = caps.at(1).unwrap().to_string();
            let value = caps.at(2).unwrap().to_string();

            info.insert(key, value);
        }
        // println!("{:?}", info);

        let fen = match info.get(&format!("{:?}", InfoKey::FEN)) {
            Some(value) => value.split_once(" ").unwrap().0,
            None => board::FEN,
        };
        let manual_move = manual_move::ManualMove::from_string(fen, manual_move_str, record_type)?;

        Ok(Manual { info, manual_move })
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
    use super::*;
    use crate::{database, evaluation};

    #[test]
    fn test_manual() {
        let manual = Manual::new();
        assert_eq!("\n\n", manual.to_string(coord::RecordType::Txt));

        let file_name_manual_strings = [
            ("01","[Atype: 残局]
[Author: ]
[Black: ]
[Date: ]
[FEN: 5a3/4ak2r/6R2/8p/9/9/9/B4N2B/4K4/3c5 r - - 0 1]
[Game: ]
[Opening: ]
[Red: ]
[Site: ]
[Title: 第01局]
[Version: 18]
[Win: 红胜]
[Writer: ]

(2)
(7,5)(5,6){从相肩进马是取胜的正确途径。其它着法，均不能取胜。}(4)
(7,5)(5,4)(1)
(9,3)(0,3)(1)
(9,3)(2,3)(1)
(1,4)(2,5)(1)
(3,8)(4,8)(1)
(9,3)(0,3)(1)
(5,6)(4,4){不怕黑炮平中拴链，进观的攻势含蓄双有诱惑性，是红方制胜的关键。}(1)
(5,6)(4,4)(2)
(5,6)(3,7){叫杀得车。}
(5,6)(4,4)(1)
(5,4)(4,6)(1)
(0,3)(0,4)(1)
(2,3)(2,4)(1)
(1,8)(1,7)(1)
(1,8)(3,8)(1)
(0,3)(0,4)(1)
(2,6)(2,5){弃车，与前着相联系，由此巧妙成杀。}(1)
(4,4)(3,6)
(2,6)(2,5)(1)
(2,6)(1,6)
(8,4)(8,3)(1)
(1,4)(2,5)(1)
(1,4)(2,5)(2)
(1,8)(1,7)(1)
(4,4)(3,6)
(4,4)(3,6)
(4,4)(2,3)
(4,6)(2,7)(1)
(1,7)(2,7)(1)
(2,6)(2,7)(1)
(1,4)(0,3)(1)
(2,7)(1,7)(1)
(1,5)(2,5){至此，形成少见的高将底炮双士和单车的局面。}(1)
(1,7)(3,7)(1)
(0,5)(1,4)(1)
(3,7)(3,5)(1)
(2,5)(2,4)(1)
(3,5)(3,8)(1)
(2,4)(2,5)(1)
(3,8)(3,5)(1)
(2,5)(2,4)(1)
(8,3)(9,3)(1)
(0,4)(0,5){和棋。}
"),
            ("4四量拨千斤","[Atype: 全局]
[Author: 橘子黄了]
[Black: ]
[Date: ]
[FEN: rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR r - - 0 1]
[Game: ]
[Opening: ]
[Red: ]
[Site: ]
[Title: 四量拨千斤]
[Version: 10]
[Win: 未知]
[Writer: 阎文清 张强]

(1)
(7,7)(7,4)(1)
(0,1)(2,2)(1)
(9,7)(7,6)(1)
(2,7)(2,5)(1)
(9,8)(9,7)(1)
(0,7)(2,6)(1)
(9,1)(7,0)(1)
(3,6)(4,6){红方左马屯边，是一种老式的攻法，优点是大子出动速度较快，不利之处是双马位置欠佳，易成弱点。

黑方挺卒活马，应着稳正，是对付红方边马阵形的有效战术。}(1)
(7,1)(7,2){平炮七线，意在加强对黑方3路线的压力，但阵营不够稳固，容易给黑方提供骚扰反击的机会。如改走炮八平六，则相对来讲要稳健一些。}(1)
(2,6)(4,5){黑方迅即跃马，有随时马6进4入侵的手段，是一种牵制战术。此外另有车1平2的选择，以下形成车九平八，炮2进4，车二进六，各攻一翼之势。}(1)
(9,0)(9,1)(1)
(2,1)(2,0){当然之着，可摆脱红车的牵制。如果走车1平2，则车八进四，黑方因单马护守中卒而不能炮2平1自然邀兑。红方占优。}(1)
(6,6)(5,6){带有欺骗性的弃兵，意在强行打开局面，实施快攻战术。通常红方多走车八进四或车二进四。}(1)
(4,6)(5,6){黑方去兵，当仁不让。如改走马6进4，红将兵三进一！马4进3，车八进二，炮6进5，马三进四，黑方得子受攻，形势不利。}(1)
(9,1)(5,1){如图形势，面对红方的捉卒，黑方主要有两种应着：（甲）卒7进1；（乙）卒7平8。现分述如下：}(2)
(5,6)(6,6){冲卒捉马，看起来是一步绝对先手，但却流于习俗，正为红方所算。}(1)
(5,6)(5,7){平卒拦车，意在延缓红方攻势，取舍异常果断，有“四两拨千斤”之妙！}(1)
(9,7)(4,7){！
进车捉马，战术紧逼，乃预谋的攻着。}(1)
(7,6)(5,7)(1)
(6,6)(7,6){另有两种选择：(1)马6退7，车二平三，车9进2，车三退二，红方主动；(2)马6退5，马三退一，黑方虽有一卒过河，但阵形呆滞，红方占有主动。}(1)
(0,8)(0,7){佳着，可顺势抢先。}(1)
(4,7)(4,5)(1)
(9,7)(7,7){高车生根，可立即迫兑黑方河口马，着法及时，否则纠缠下去于红方无益。}(1)
(0,3)(1,4)(1)
(0,7)(5,7)(1)
(6,2)(5,2){依仗出子优势，红方继续贯彻强攻计划。若改走炮七平三，则象3进5，局面较为平稳，红方略占先手。}(1)
(7,7)(5,7)(1)
(0,2)(2,4)(1)
(4,5)(5,7)(1)
(5,2)(4,2){！}(1)
(5,1)(5,7)(1)
(3,2)(4,2){对黑方消极的象5进3，红有马九进七下伏马七进六或马七进五等手段，将全线出击。}(1)
(0,2)(2,4){经过转换，烟消云散，双方趋于平稳。}(1)
(7,2)(2,2)(1)
(6,0)(5,0)(1)
(2,5)(2,2)(1)
(0,3)(1,4){补士固防，稳正之着，当然不宜走卒3进1，否则红将兵七进一乘势进攻。}(1)
(7,4)(3,4)(1)
(7,2)(3,2)(1)
(2,2)(9,2)(1)
(3,8)(4,8){细致的一手，不给红方炮七平一打卒的机会。}(1)
(9,3)(8,4){红方持有中炮攻势，占有优势。}
(7,0)(5,1)(1)
(0,0)(0,3){双方大致均势。


［小结］对于红方所施的骗着，黑方（甲）变不够明智，遭到了红方的猛攻，处境不妙。（乙）变黑方妙用平卒巧着，有效地遏制了红方攻势，双方平分秋色。

在本局中。红方的布局骗着具有快速突击的特点。对此，黑方愈是用强，红势则愈旺。黑若能冷静对待，并采取（乙）变着法，延缓红势的策略，可安然无恙。}
"),
            ("第09局","[Atype: 残局]
[Author: ]
[Black: ]
[Date: ]
[FEN: 5k3/9/9/9/9/9/4rp3/2R1C4/4K4/9 r - - 0 1]
[Game: ]
[Opening: ]
[Red: ]
[Site: ]
[Title: 第09局]
[Version: 18]
[Win: 红胜]
[Writer: ]

{这是一局炮斗车卒的范例。对车炮的运用颇有启迪，可资借鉴。}(1)
(7,2)(5,2)(3)
(6,4)(6,0)(2)
(6,4)(6,3)(1)
(6,4)(1,4)(1)
(7,4)(7,5){献炮叫将，伏车八平四白脸将成杀，是获胜的关键。}(1)
(7,4)(7,7)(1)
(5,2)(5,4){红车占中是获胜的休着。黑不敢平车邀兑，否则，红车五平四胜。}(1)
(5,2)(5,7)(1)
(6,5)(6,4)(1)
(6,0)(6,4){将军。}(1)
(0,5)(1,5)(1)
(1,4)(1,6)(1)
(5,2)(5,5)(1)
(7,7)(7,4){叫杀。}(1)
(7,4)(7,5)(1)
(5,7)(0,7){红方升车再打将，使黑方车卒失去有机联系，是获胜的重要环节。}(1)
(0,5)(0,4)(1)
(6,4)(6,0)(1)
(6,5)(6,4)(1)
(0,5)(1,5)(1)
(5,5)(5,4)(2)
(7,4)(7,7){“二打对一打”，红方不变作负。}
(7,5)(7,6)(1)
(0,7)(6,7)(3)
(0,4)(0,5)(1)
(0,4)(0,3)(1)
(1,5)(0,5)(1)
(6,5)(6,6)(1)
(1,6)(6,6)(1)
(6,5)(7,5)(1)
(7,5)(7,6)(1)
(7,5)(6,5)(1)
(7,6)(6,6)(1)
(6,7)(5,7)(1)
(7,4)(7,5)(1)
(6,7)(6,5)(1)
(0,5)(1,5)(1)
(6,0)(8,0)(1)
(6,4)(7,4)(1)
(1,5)(0,5)(1)
(6,5)(7,5)(1)
(1,5)(1,4)(1)
(7,6)(6,6)(1)
(8,4)(9,4)(1)
(5,4)(7,4){红方胜定。}
(5,7)(5,5)(1)
(6,7)(6,6)
(6,5)(7,5){以下升车占中，海底捞月胜。}
(6,0)(8,0){平炮再升炮打车，消灭小卒，催毁黑方中路屏障，是红方获胜的精华。}(1)
(8,0)(8,3)(1)
(0,5)(0,4)(1)
(8,4)(9,4)(1)
(5,4)(6,4)(1)
(5,5)(5,4)(1)
(8,0)(8,5)(1)
(8,3)(7,3)(1)
(0,4)(0,3)(1)
(5,4)(6,4)(1)
(6,4)(0,4)(1)
(7,4)(7,3){红方胜定。}
(8,5)(7,5)(1)
(0,3)(1,3)(1)
(6,6)(0,6)(1)
(6,5)(6,2){以下海底捞月红胜。}
(7,5)(2,5)(1)
(6,4)(1,4)(1)
(1,5)(0,5)(1)
(1,4)(0,4)(1)
(0,5)(1,5)(1)
(0,6)(0,5)(1)
(2,5)(2,6)(1)
(0,4)(4,4)(1)
(1,5)(0,5)(1)
(4,4)(4,5)
"),
            ("布局陷阱--飞相局对金钩炮","[Atype: 全局]
[Author: ]
[Black: ]
[Date: ]
[FEN: rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR r - - 0 1]
[Game: 布局陷阱--飞相局对金钩炮]
[Opening: ]
[Red: ]
[Site: ]
[Title: 布局陷阱--飞相局对金钩炮]
[Version: 12]
[Win: 红胜]
[Writer: ]

(1)
(9,6)(7,4)(1)
(2,7)(2,2)(1)
(9,8)(8,8)(1)
(0,7)(2,6)(1)
(8,8)(8,3)(1)
(0,8)(0,7)(1)
(8,3)(1,3)(1)
(0,1)(2,0)(1)
(1,3)(1,1)(1)
(2,1)(9,1)(1)
(9,0)(9,1)(1)
(0,6)(2,4)(1)
(7,1)(7,0)(1)
(0,5)(1,4)(1)
(9,1)(2,1)(1)
(2,2)(2,3)(1)
(9,7)(8,5)(1)
(0,7)(4,7)(1)
(6,0)(5,0)(1)
(2,3)(3,3)(1)
(1,1)(1,3)(1)
(3,3)(2,3)(1)
(7,0)(3,0)(1)
(2,0)(0,1)(1)
(3,0)(4,0)(1)
(4,7)(4,5)(1)
(8,5)(9,7)(1)
(3,6)(4,6)(1)
(1,3)(1,1){红得子大优}
"),
            ("- 北京张强 (和) 上海胡荣华 (1993.4.27于南京)","[Atype: 全局]
[Author: ]
[Black: 上海胡荣华]
[Date: 1993.4.27]
[FEN: rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR r - - 0 1]
[Game: 93全国象棋锦标赛]
[Opening: ]
[Red: 北京张强]
[Site: 南京]
[Title: 挺兵对卒底炮]
[Version: 13]
[Win: 和棋]
[Writer: ]

(1)
(6,2)(5,2)(1)
(2,1)(2,2)(1)
(7,7)(7,4)(1)
(0,2)(2,4)(1)
(9,7)(7,6)(1)
(3,2)(4,2)(1)
(9,1)(7,0)(1)
(4,2)(5,2)(1)
(9,8)(9,7)(1)
(0,8)(1,8)(1)
(9,0)(9,1)(1)
(1,8)(1,3)(1)
(9,3)(8,4)(1)
(0,3)(1,4)(1)
(9,7)(5,7)(1)
(3,6)(4,6)(1)
(5,7)(5,2)(1)
(0,7)(2,6)(1)
(6,6)(5,6)(1)
(4,6)(5,6)(1)
(5,2)(5,6)(1)
(3,0)(4,0)(1)
(7,1)(7,3)(1)
(0,1)(2,0)(1)
(9,1)(2,1)(1)
(2,6)(4,5)(1)
(2,1)(3,1)(1)
(1,3)(5,3)(1)
(5,6)(5,5)(1)
(2,7)(2,6)(1)
(9,6)(7,8)(1)
(5,3)(4,3)(1)
(7,4)(3,4)(1)
(4,3)(4,2)(1)
(9,2)(7,4)(1)
(0,0)(0,1)(1)
(3,1)(0,1)(1)
(2,0)(0,1)(1)
(7,6)(5,7)(1)
(4,5)(5,7)(1)
(5,5)(5,7)(1)
(2,2)(0,2)(1)
(5,7)(5,6)(1)
(0,2)(2,2)(1)
(6,0)(5,0)(1)
(0,1)(1,3)(1)
(3,4)(3,5)(1)
(4,2)(4,5)(1)
(3,5)(3,7)(1)
(1,3)(3,4)(1)
(5,6)(5,1)(1)
(4,0)(5,0)(1)
(5,1)(5,0)(1)
(3,4)(4,2)(1)
(5,0)(0,0)(1)
(2,2)(0,2)(1)
(7,0)(6,2)(1)
(4,5)(4,4)(1)
(6,4)(5,4)(1)
(4,2)(5,4)
"),
        ];

        fn get_file_path(file_name: &str, record_type: coord::RecordType) -> String {
            format!("tests/output/{}.{}", file_name, record_type.ext_name())
        }

        let mut filename_manuals = Vec::<(&str, Manual)>::new();
        let mut zorbist_evaluation = evaluation::ZorbistEvaluation::new();
        for (file_name, manual_string) in file_name_manual_strings {
            if let Ok(manual) = Manual::from(&format!("tests/xqf/{file_name}.xqf")) {
                assert_eq!(manual_string, manual.to_string(coord::RecordType::Txt));
                zorbist_evaluation.append(manual.manual_move.get_zorbist_evaluation());

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
                    // zorbist_aspect_evaluation
                    //     .append(manual.manual_move.get_zorbist_aspect_evaluation());

                    if let Ok(manual) = Manual::from(&file_path) {
                        assert_eq!(manual_string, manual.to_string(coord::RecordType::Txt));
                    }
                }
                filename_manuals.push((file_name, manual));
            }
        }

        let result = zorbist_evaluation.to_string();
        std::fs::write(format!("tests/output/zobrist_evaluation.txt"), result).expect("Write Err.");

        let json_file_name = "tests/output/serde_json.txt";
        let result = serde_json::to_string(&zorbist_evaluation).unwrap();
        std::fs::write(json_file_name, result).expect("Write Err.");

        // serde_json
        let vec_u8 = std::fs::read(json_file_name).unwrap();
        let zorbist_eval: evaluation::ZorbistEvaluation =
            serde_json::from_str(&String::from_utf8(vec_u8).unwrap()).unwrap();
        let result = zorbist_eval.to_string();
        std::fs::write(format!("tests/output/zobrist_eval.txt"), result).expect("Write Err.");

        // database
        let mut conn = database::get_connection();
        let _ = database::insert_manuals(&mut conn, filename_manuals)
            .map_err(|err| assert!(false, "insert_manuals: {:?}!\n", err));
        let _ = database::init_zorbist_evaluation(&mut conn, &zorbist_evaluation)
            .map_err(|err| assert!(false, "insert_zorbist_evaluation: {:?}!\n", err));
    }
}
