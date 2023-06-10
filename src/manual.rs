#![allow(dead_code)]

use encoding::all::GBK;
use encoding::{DecoderTrap, Encoding};
// use crate::amove;
use crate::bit_constant::{COLCOUNT, ROWCOUNT, SEATCOUNT};
use crate::manual_move;
// use crate::bit_constant;
use crate::board;
use std::collections::BTreeMap;
// use std::rc::Rc;

#[derive(Debug)]
pub enum FileExtType {
    Xqf,
    Bin,
    Txt,
    PgnRc,
    PgnIccs,
    PgnZh,
}

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

    fn from_xqf(file_name: &str) -> Self {
        let mut info = BTreeMap::new();
        let mut manual_move = manual_move::ManualMove::new();
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
                &fen, &byte_vec, version, keyxyf, keyxyt, keyrmksize, &f32keys,
            );
        }

        Manual { info, manual_move }
    }

    fn set_infokey_value(&mut self, key: String, value: String) {
        self.info.insert(key, value);
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (key, value) in &self.info {
            result.push_str(&format!("[{key}: {value}]\n"));
        }
        result.push('\n');
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
        assert_eq!("\n", manual.to_string());

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

  0_  1[(7,5)->(5,6)]{从相肩进马是取胜的正确途径。其它着法，均不能取胜。}
  1_  2[(9,3)->(0,3)]
  2_  3[(5,6)->(4,4)]{不怕黑炮平中拴链，进观的攻势含蓄双有诱惑性，是红方制胜的关键。}
  3_  4[(0,3)->(0,4)]
  4_  5[(2,6)->(2,5)]{弃车，与前着相联系，由此巧妙成杀。}
  5_  6[(1,4)->(2,5)]
  6_  7[(4,4)->(3,6)]
  1_  8[(9,3)->(2,3)]
  8_  9[(5,6)->(4,4)]
  9_ 10[(2,3)->(2,4)]
 10_ 11[(4,4)->(3,6)]
  9_ 12[(1,8)->(1,7)]
 12_ 13[(2,6)->(2,5)]
 13_ 14[(1,4)->(2,5)]
 14_ 15[(4,4)->(3,6)]
 14_ 16[(4,4)->(2,3)]
  1_ 17[(1,4)->(2,5)]
 17_ 18[(5,6)->(3,7)]{叫杀得车。}
  1_ 19[(3,8)->(4,8)]
 19_ 20[(5,6)->(4,4)]
 20_ 21[(1,8)->(3,8)]
 21_ 22[(2,6)->(1,6)]
  0_ 23[(7,5)->(5,4)]
 23_ 24[(9,3)->(0,3)]
 24_ 25[(5,4)->(4,6)]
 25_ 26[(0,3)->(0,4)]
 26_ 27[(8,4)->(8,3)]
 27_ 28[(1,8)->(1,7)]
 28_ 29[(4,6)->(2,7)]
 29_ 30[(1,7)->(2,7)]
 30_ 31[(2,6)->(2,7)]
 31_ 32[(1,4)->(0,3)]
 32_ 33[(2,7)->(1,7)]
 33_ 34[(1,5)->(2,5)]{至此，形成少见的高将底炮双士和单车的局面。}
 34_ 35[(1,7)->(3,7)]
 35_ 36[(0,5)->(1,4)]
 36_ 37[(3,7)->(3,5)]
 37_ 38[(2,5)->(2,4)]
 38_ 39[(3,5)->(3,8)]
 39_ 40[(2,4)->(2,5)]
 40_ 41[(3,8)->(3,5)]
 41_ 42[(2,5)->(2,4)]
 42_ 43[(8,3)->(9,3)]
 43_ 44[(0,4)->(0,5)]{和棋。}
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

  0_  1[(7,7)->(7,4)]
  1_  2[(0,1)->(2,2)]
  2_  3[(9,7)->(7,6)]
  3_  4[(2,7)->(2,5)]
  4_  5[(9,8)->(9,7)]
  5_  6[(0,7)->(2,6)]
  6_  7[(9,1)->(7,0)]
  7_  8[(3,6)->(4,6)]{红方左马屯边，是一种老式的攻法，优点是大子出动速度较快，不利之处是双马位置欠佳，易成弱点。

黑方挺卒活马，应着稳正，是对付红方边马阵形的有效战术。}
  8_  9[(7,1)->(7,2)]{平炮七线，意在加强对黑方3路线的压力，但阵营不够稳固，容易给黑方提供骚扰反击的机会。如改走炮八平六，则相对来讲要稳健一些。}
  9_ 10[(2,6)->(4,5)]{黑方迅即跃马，有随时马6进4入侵的手段，是一种牵制战术。此外另有车1平2的选择，以下形成车九平八，炮2进4，车二进六，各攻一翼之势。}
 10_ 11[(9,0)->(9,1)]
 11_ 12[(2,1)->(2,0)]{当然之着，可摆脱红车的牵制。如果走车1平2，则车八进四，黑方因单马护守中卒而不能炮2平1自然邀兑。红方占优。}
 12_ 13[(6,6)->(5,6)]{带有欺骗性的弃兵，意在强行打开局面，实施快攻战术。通常红方多走车八进四或车二进四。}
 13_ 14[(4,6)->(5,6)]{黑方去兵，当仁不让。如改走马6进4，红将兵三进一！马4进3，车八进二，炮6进5，马三进四，黑方得子受攻，形势不利。}
 14_ 15[(9,1)->(5,1)]{如图形势，面对红方的捉卒，黑方主要有两种应着：（甲）卒7进1；（乙）卒7平8。现分述如下：}
 15_ 16[(5,6)->(6,6)]{冲卒捉马，看起来是一步绝对先手，但却流于习俗，正为红方所算。}
 16_ 17[(9,7)->(4,7)]{！
进车捉马，战术紧逼，乃预谋的攻着。}
 17_ 18[(6,6)->(7,6)]{另有两种选择：(1)马6退7，车二平三，车9进2，车三退二，红方主动；(2)马6退5，马三退一，黑方虽有一卒过河，但阵形呆滞，红方占有主动。}
 18_ 19[(4,7)->(4,5)]
 19_ 20[(0,3)->(1,4)]
 20_ 21[(6,2)->(5,2)]{依仗出子优势，红方继续贯彻强攻计划。若改走炮七平三，则象3进5，局面较为平稳，红方略占先手。}
 21_ 22[(0,2)->(2,4)]
 22_ 23[(5,2)->(4,2)]{！}
 23_ 24[(3,2)->(4,2)]{对黑方消极的象5进3，红有马九进七下伏马七进六或马七进五等手段，将全线出击。}
 24_ 25[(7,2)->(2,2)]
 25_ 26[(2,5)->(2,2)]
 26_ 27[(7,4)->(3,4)]
 27_ 28[(2,2)->(9,2)]
 28_ 29[(9,3)->(8,4)]{红方持有中炮攻势，占有优势。}
 15_ 30[(5,6)->(5,7)]{平卒拦车，意在延缓红方攻势，取舍异常果断，有“四两拨千斤”之妙！}
 30_ 31[(7,6)->(5,7)]
 31_ 32[(0,8)->(0,7)]{佳着，可顺势抢先。}
 32_ 33[(9,7)->(7,7)]{高车生根，可立即迫兑黑方河口马，着法及时，否则纠缠下去于红方无益。}
 33_ 34[(0,7)->(5,7)]
 34_ 35[(7,7)->(5,7)]
 35_ 36[(4,5)->(5,7)]
 36_ 37[(5,1)->(5,7)]
 37_ 38[(0,2)->(2,4)]{经过转换，烟消云散，双方趋于平稳。}
 38_ 39[(6,0)->(5,0)]
 39_ 40[(0,3)->(1,4)]{补士固防，稳正之着，当然不宜走卒3进1，否则红将兵七进一乘势进攻。}
 40_ 41[(7,2)->(3,2)]
 41_ 42[(3,8)->(4,8)]{细致的一手，不给红方炮七平一打卒的机会。}
 42_ 43[(7,0)->(5,1)]
 43_ 44[(0,0)->(0,3)]{双方大致均势。


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

{这是一局炮斗车卒的范例。对车炮的运用颇有启迪，可资借鉴。}
  0_  1[(7,2)->(5,2)]
  1_  2[(6,4)->(6,0)]
  2_  3[(7,4)->(7,5)]{献炮叫将，伏车八平四白脸将成杀，是获胜的关键。}
  3_  4[(6,5)->(6,4)]
  4_  5[(5,2)->(5,5)]
  5_  6[(0,5)->(0,4)]
  6_  7[(5,5)->(5,4)]
  7_  8[(0,4)->(0,5)]
  8_  9[(7,5)->(7,6)]
  9_ 10[(0,5)->(1,5)]
 10_ 11[(7,6)->(6,6)]
 11_ 12[(6,0)->(8,0)]{平炮再升炮打车，消灭小卒，催毁黑方中路屏障，是红方获胜的精华。}
 12_ 13[(8,4)->(9,4)]
 13_ 14[(8,0)->(8,5)]
 14_ 15[(5,4)->(6,4)]
 15_ 16[(8,5)->(7,5)]
 16_ 17[(6,6)->(0,6)]
 17_ 18[(7,5)->(2,5)]
 18_ 19[(6,4)->(1,4)]
 19_ 20[(1,5)->(0,5)]
 20_ 21[(1,4)->(0,4)]
 21_ 22[(0,5)->(1,5)]
 22_ 23[(0,6)->(0,5)]
 23_ 24[(2,5)->(2,6)]
 24_ 25[(0,4)->(4,4)]
 25_ 26[(1,5)->(0,5)]
 26_ 27[(4,4)->(4,5)]
  7_ 28[(0,4)->(0,3)]
 28_ 29[(7,5)->(6,5)]
 29_ 30[(6,0)->(8,0)]
 30_ 31[(8,4)->(9,4)]
 31_ 32[(8,0)->(8,3)]
 32_ 33[(5,4)->(6,4)]
 33_ 34[(8,3)->(7,3)]
 34_ 35[(6,4)->(0,4)]
 35_ 36[(0,3)->(1,3)]
 36_ 37[(6,5)->(6,2)]{以下海底捞月红胜。}
  2_ 38[(7,4)->(7,7)]
 38_ 39[(6,0)->(6,4)]{将军。}
 39_ 40[(7,7)->(7,4)]{叫杀。}
 40_ 41[(6,4)->(6,0)]
 41_ 42[(7,4)->(7,7)]{“二打对一打”，红方不变作负。}
  1_ 43[(6,4)->(6,3)]
 43_ 44[(5,2)->(5,4)]{红车占中是获胜的休着。黑不敢平车邀兑，否则，红车五平四胜。}
 44_ 45[(0,5)->(1,5)]
 45_ 46[(7,4)->(7,5)]
 46_ 47[(6,5)->(6,4)]
 47_ 48[(7,5)->(7,6)]
 48_ 49[(1,5)->(0,5)]
 49_ 50[(7,6)->(6,6)]
 50_ 51[(6,4)->(7,4)]
 51_ 52[(5,4)->(7,4)]{红方胜定。}
  1_ 53[(6,4)->(1,4)]
 53_ 54[(5,2)->(5,7)]
 54_ 55[(1,4)->(1,6)]
 55_ 56[(5,7)->(0,7)]{红方升车再打将，使黑方车卒失去有机联系，是获胜的重要环节。}
 56_ 57[(0,5)->(1,5)]
 57_ 58[(0,7)->(6,7)]
 58_ 59[(6,5)->(6,6)]
 59_ 60[(6,7)->(5,7)]
 60_ 61[(1,5)->(0,5)]
 61_ 62[(5,7)->(5,5)]
 62_ 63[(0,5)->(0,4)]
 63_ 64[(5,5)->(5,4)]
 64_ 65[(0,4)->(0,3)]
 65_ 66[(7,4)->(7,3)]{红方胜定。}
 58_ 67[(1,6)->(6,6)]
 67_ 68[(7,4)->(7,5)]
 68_ 69[(6,5)->(7,5)]
 69_ 70[(6,7)->(6,6)]
 58_ 71[(6,5)->(7,5)]
 71_ 72[(6,7)->(6,5)]
 72_ 73[(1,5)->(1,4)]
 73_ 74[(6,5)->(7,5)]{以下升车占中，海底捞月胜。}
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

  0_  1[(9,6)->(7,4)]
  1_  2[(2,7)->(2,2)]
  2_  3[(9,8)->(8,8)]
  3_  4[(0,7)->(2,6)]
  4_  5[(8,8)->(8,3)]
  5_  6[(0,8)->(0,7)]
  6_  7[(8,3)->(1,3)]
  7_  8[(0,1)->(2,0)]
  8_  9[(1,3)->(1,1)]
  9_ 10[(2,1)->(9,1)]
 10_ 11[(9,0)->(9,1)]
 11_ 12[(0,6)->(2,4)]
 12_ 13[(7,1)->(7,0)]
 13_ 14[(0,5)->(1,4)]
 14_ 15[(9,1)->(2,1)]
 15_ 16[(2,2)->(2,3)]
 16_ 17[(9,7)->(8,5)]
 17_ 18[(0,7)->(4,7)]
 18_ 19[(6,0)->(5,0)]
 19_ 20[(2,3)->(3,3)]
 20_ 21[(1,1)->(1,3)]
 21_ 22[(3,3)->(2,3)]
 22_ 23[(7,0)->(3,0)]
 23_ 24[(2,0)->(0,1)]
 24_ 25[(3,0)->(4,0)]
 25_ 26[(4,7)->(4,5)]
 26_ 27[(8,5)->(9,7)]
 27_ 28[(3,6)->(4,6)]
 28_ 29[(1,3)->(1,1)]{红得子大优}
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

  0_  1[(6,2)->(5,2)]
  1_  2[(2,1)->(2,2)]
  2_  3[(7,7)->(7,4)]
  3_  4[(0,2)->(2,4)]
  4_  5[(9,7)->(7,6)]
  5_  6[(3,2)->(4,2)]
  6_  7[(9,1)->(7,0)]
  7_  8[(4,2)->(5,2)]
  8_  9[(9,8)->(9,7)]
  9_ 10[(0,8)->(1,8)]
 10_ 11[(9,0)->(9,1)]
 11_ 12[(1,8)->(1,3)]
 12_ 13[(9,3)->(8,4)]
 13_ 14[(0,3)->(1,4)]
 14_ 15[(9,7)->(5,7)]
 15_ 16[(3,6)->(4,6)]
 16_ 17[(5,7)->(5,2)]
 17_ 18[(0,7)->(2,6)]
 18_ 19[(6,6)->(5,6)]
 19_ 20[(4,6)->(5,6)]
 20_ 21[(5,2)->(5,6)]
 21_ 22[(3,0)->(4,0)]
 22_ 23[(7,1)->(7,3)]
 23_ 24[(0,1)->(2,0)]
 24_ 25[(9,1)->(2,1)]
 25_ 26[(2,6)->(4,5)]
 26_ 27[(2,1)->(3,1)]
 27_ 28[(1,3)->(5,3)]
 28_ 29[(5,6)->(5,5)]
 29_ 30[(2,7)->(2,6)]
 30_ 31[(9,6)->(7,8)]
 31_ 32[(5,3)->(4,3)]
 32_ 33[(7,4)->(3,4)]
 33_ 34[(4,3)->(4,2)]
 34_ 35[(9,2)->(7,4)]
 35_ 36[(0,0)->(0,1)]
 36_ 37[(3,1)->(0,1)]
 37_ 38[(2,0)->(0,1)]
 38_ 39[(7,6)->(5,7)]
 39_ 40[(4,5)->(5,7)]
 40_ 41[(5,5)->(5,7)]
 41_ 42[(2,2)->(0,2)]
 42_ 43[(5,7)->(5,6)]
 43_ 44[(0,2)->(2,2)]
 44_ 45[(6,0)->(5,0)]
 45_ 46[(0,1)->(1,3)]
 46_ 47[(3,4)->(3,5)]
 47_ 48[(4,2)->(4,5)]
 48_ 49[(3,5)->(3,7)]
 49_ 50[(1,3)->(3,4)]
 50_ 51[(5,6)->(5,1)]
 51_ 52[(4,0)->(5,0)]
 52_ 53[(5,1)->(5,0)]
 53_ 54[(3,4)->(4,2)]
 54_ 55[(5,0)->(0,0)]
 55_ 56[(2,2)->(0,2)]
 56_ 57[(7,0)->(6,2)]
 57_ 58[(4,5)->(4,4)]
 58_ 59[(6,4)->(5,4)]
 59_ 60[(4,2)->(5,4)]
"),
        ];
        for (file_name, manual_string) in file_name_manual_strings {
            let manual = Manual::from_xqf(&format!("tests/xqf/{file_name}.xqf"));

            assert_eq!(manual_string, manual.to_string());
            //     std::fs::write(format!("tests/{file_name}.txt"), manual.to_string())
            //         .expect("Write Err.");
        }
    }
}
