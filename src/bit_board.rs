#![allow(dead_code)]
#![allow(unused_imports)]

use crate::bit_constant;
use crate::bit_effect;
use crate::board;
use crate::coord::{
    self, COLCOUNT, COLSTATECOUNT, LEGSTATECOUNT, ROWCOUNT, ROWSTATECOUNT, SEATCOUNT, SIDECOUNT,
};
use crate::piece::{self, COLORCOUNT, KINDCOUNT};

type SetEffect =
    fn(&BitBoard, move_effect: &mut bit_effect::MoveEffect, to_index: usize, eat_kind: piece::Kind);

#[derive(Debug)]
pub struct BitBoard {
    bottom_color: piece::Color,
    kinds: [piece::Kind; SEATCOUNT],

    // 计算中间存储数据(基本局面改动时更新)
    color_kind_pieces: [[bit_constant::BitAtom; KINDCOUNT]; COLORCOUNT],
    color_pieces: [bit_constant::BitAtom; COLORCOUNT],
    all_pieces: bit_constant::BitAtom,
    rotate_all_pieces: bit_constant::BitAtom,

    // 哈希局面数据
    hashkey: u64,
    hashlock: u64,
    // private static HistoryRecord? historyRecord;
}

impl BitBoard {
    pub fn new(pieces: &board::Pieces) -> BitBoard {
        let mut bit_board: BitBoard = BitBoard {
            bottom_color: board::get_bottom_color(pieces),
            kinds: [piece::Kind::NoKind; SEATCOUNT],

            color_kind_pieces: [[0; KINDCOUNT]; COLORCOUNT],
            color_pieces: [0; COLORCOUNT],
            all_pieces: 0,
            rotate_all_pieces: 0,

            hashkey: 0,
            hashlock: 0,
        };

        for (index, piece) in pieces.iter().enumerate() {
            if let piece::Piece::Some(color, kind) = piece {
                let color_i = *color as usize;
                let kind_i = *kind as usize;
                bit_board.kinds[index] = *kind;

                bit_board.color_kind_pieces[color_i][kind_i] |= bit_constant::MASK[index];
                bit_board.color_pieces[color_i] |= bit_constant::MASK[index];
                bit_board.all_pieces |= bit_constant::MASK[index];
                bit_board.rotate_all_pieces |= bit_constant::ROTATEMASK[index];

                bit_board.hashkey ^= bit_constant::ZOBRISTKEY[color_i][kind_i][index];
                bit_board.hashlock ^= bit_constant::ZOBRISTLOCK[color_i][kind_i][index];
            }
        }

        bit_board
    }

    pub fn get_hash_key(&self, color: piece::Color) -> u64 {
        self.hashkey ^ bit_constant::COLORZOBRISTKEY[color as usize]
    }

    pub fn get_hash_lock(&self, color: piece::Color) -> u64 {
        self.hashlock ^ bit_constant::COLORZOBRISTLOCK[color as usize]
    }

    fn get_color(&self, index: usize) -> piece::Color {
        if self.color_pieces[piece::Color::Red as usize] & bit_constant::MASK[index] == 0 {
            piece::Color::Black
        } else {
            piece::Color::Red
        }
    }

    fn get_move_from_index(&self, index: usize) -> bit_constant::BitAtom {
        let color = self.get_color(index);
        let kind = self.kinds[index];
        let result = match kind {
            piece::Kind::King => bit_constant::KINGMOVE[index],
            piece::Kind::Advisor => bit_constant::ADVISORMOVE[index],
            piece::Kind::Bishop => bit_constant::get_bishop_move(index, self.all_pieces),
            piece::Kind::Knight => bit_constant::get_knight_move(index, self.all_pieces),
            piece::Kind::Rook => {
                bit_constant::get_rook_move(index, self.all_pieces, self.rotate_all_pieces)
            }
            piece::Kind::Cannon => {
                bit_constant::get_cannon_move(index, self.all_pieces, self.rotate_all_pieces)
            }
            piece::Kind::Pawn => bit_constant::get_pawn_move(color == self.bottom_color, index),
            _ => 0,
        };

        // 去掉同色棋子
        result ^ (result & self.color_pieces[color as usize])
    }

    fn get_move_from_bitatom(&self, bit_atom: bit_constant::BitAtom) -> bit_constant::BitAtom {
        let mut result = 0;
        for index in bit_constant::get_indexs_from_bitatom(bit_atom) {
            result |= self.get_move_from_index(index);
        }

        result
    }

    fn get_move_from_color_kind(
        &self,
        color: piece::Color,
        kind: piece::Kind,
    ) -> bit_constant::BitAtom {
        self.get_move_from_bitatom(self.color_kind_pieces[color as usize][kind as usize])
    }

    fn get_move_from_color(&self, color: piece::Color) -> bit_constant::BitAtom {
        self.get_move_from_bitatom(self.color_pieces[color as usize])
    }

    pub fn is_killed(&self, color: piece::Color) -> bool {
        let other_color = piece::other_color(color);
        let king_bitatom = self.color_kind_pieces[color as usize][piece::Kind::King as usize];
        let otherking_bitatom =
            self.color_kind_pieces[other_color as usize][piece::Kind::King as usize];
        let king_face = || {
            let king_indexs =
                bit_constant::get_indexs_from_bitatom(king_bitatom | otherking_bitatom);
            assert_eq!(king_indexs.len(), 2);

            let top_king_index = king_indexs[0];
            let bottom_king_index = king_indexs[1];
            if !crate::is_same_col!(top_king_index, bottom_king_index) {
                return false;
            }

            let mut index = top_king_index + COLCOUNT;
            while index < bottom_king_index {
                if self.all_pieces & bit_constant::MASK[index] != 0 {
                    return false;
                }
                index += COLCOUNT;
            }

            true
        };

        king_face() || (self.get_move_from_color(other_color) & king_bitatom) != 0
    }

    pub fn is_failed(&self, color: piece::Color) -> bool {
        self.get_move_from_color(color) == 0
    }

    fn do_move(
        &mut self,
        from_index: usize,
        to_index: usize,
        is_back: bool,
        mut eat_kind: piece::Kind,
    ) -> piece::Kind {
        let start_index = if is_back { to_index } else { from_index };
        let end_index = if is_back { from_index } else { to_index };
        let from_kind = self.kinds[start_index];
        let from_color_i = self.get_color(start_index) as usize;
        let from_kind_i = from_kind as usize;
        let from_bitatrom = bit_constant::MASK[from_index];
        let to_bitatom = bit_constant::MASK[to_index];
        let move_bitatom = from_bitatrom | to_bitatom;
        if !is_back {
            eat_kind = self.kinds[to_index];
        }

        // 清除原位置，置位新位置
        self.kinds[end_index] = from_kind;
        self.kinds[start_index] = piece::Kind::NoKind;

        self.color_kind_pieces[from_color_i][from_kind_i] ^= move_bitatom;
        self.color_pieces[from_color_i] ^= move_bitatom;

        // self.hashkey ^= bit_constant::ZOBRISTKEY[from_color_i][from_kind_i][from_index]
        //     ^ bit_constant::ZOBRISTKEY[from_color_i][from_kind_i][to_index];
        // self.hashlock ^= bit_constant::ZOBRISTKEY[from_color_i][from_kind_i][from_index]
        //     ^ bit_constant::ZOBRISTKEY[from_color_i][from_kind_i][to_index];

        if eat_kind != piece::Kind::NoKind {
            let to_color_i = if from_color_i == 0 { 1 } else { 0 };
            let eat_kind_i = eat_kind as usize;
            if is_back {
                self.kinds[start_index] = eat_kind;
            }
            self.color_kind_pieces[to_color_i][eat_kind_i] ^= to_bitatom;
            self.color_pieces[to_color_i] ^= to_bitatom;

            // self.hashkey ^= bit_constant::ZOBRISTKEY[to_color_i][eat_kind_i][to_index];
            // self.hashlock ^= bit_constant::ZOBRISTKEY[to_color_i][eat_kind_i][to_index];

            self.all_pieces ^= from_bitatrom;
            self.rotate_all_pieces ^= bit_constant::ROTATEMASK[from_index];
        } else {
            self.all_pieces ^= move_bitatom;
            self.rotate_all_pieces ^=
                bit_constant::ROTATEMASK[from_index] | bit_constant::ROTATEMASK[to_index];
        }

        eat_kind
    }

    fn set_effect_killed(
        &self,
        move_effect: &mut bit_effect::MoveEffect,
        to_index: usize,
        eat_kind: piece::Kind,
    ) {
        // 如是对方将帅的位置则直接可走，不用判断是否被将军（如加以判断，则会直接走棋吃将帅）；棋子已走，取终点位置颜色
        let is_killed = eat_kind != piece::Kind::King && self.is_killed(self.get_color(to_index));
        let score = if is_killed { -1 } else { 0 };
        // 扩展，增加其他功能

        move_effect.add(to_index, score, 0);
    }

    // 执行某一着后的效果(委托函数可叠加)
    fn domove_set_effect_undo_move(
        &mut self,
        move_effect: &mut bit_effect::MoveEffect,
        to_index: usize,
        set_effect: SetEffect,
    ) {
        let eat_kind = self.do_move(move_effect.from_index, to_index, false, piece::Kind::NoKind);

        set_effect(self, move_effect, to_index, eat_kind);

        self.do_move(move_effect.from_index, to_index, true, eat_kind);
    }

    fn get_effect_from_index(&mut self, from_index: usize) -> bit_effect::MoveEffect {
        let mut move_effect = bit_effect::MoveEffect::new(from_index);
        for to_index in bit_constant::get_indexs_from_bitatom(self.get_move_from_index(from_index))
        {
            self.domove_set_effect_undo_move(&mut move_effect, to_index, Self::set_effect_killed);
        }

        move_effect
    }

    fn get_effects_from_bitatom(
        &mut self,
        bit_atom: bit_constant::BitAtom,
    ) -> Vec<bit_effect::MoveEffect> {
        let mut effects: Vec<bit_effect::MoveEffect> = Vec::new();
        for from_index in bit_constant::get_indexs_from_bitatom(bit_atom) {
            effects.push(self.get_effect_from_index(from_index));
        }

        effects
    }

    // kind == piece::Kind::NoKind，取全部种类棋子
    fn get_effects_from_color_kind(
        &mut self,
        color: piece::Color,
        kind: piece::Kind,
    ) -> Vec<bit_effect::MoveEffect> {
        self.get_effects_from_bitatom(match kind {
            piece::Kind::NoKind => self.color_pieces[color as usize],
            _ => self.color_kind_pieces[color as usize][kind as usize],
        })
    }

    pub fn get_effects_from_color(&mut self, color: piece::Color) -> Vec<bit_effect::MoveEffect> {
        self.get_effects_from_color_kind(color, piece::Kind::NoKind)
    }

    // 可变借用
    pub fn to_moves_string(&mut self) -> String {
        let mut result = format!("moves_string:\n");
        for color in [piece::Color::Red, piece::Color::Black] {
            let mut moves = Vec::new();
            for index in bit_constant::get_indexs_from_bitatom(self.color_pieces[color as usize]) {
                moves.push(self.get_move_from_index(index));
            }

            result.push_str(&bit_constant::get_bitatom_array_string(&moves, false));
        }

        result
    }

    // 可变借用
    pub fn to_effects_string(&mut self) -> String {
        let mut result = format!("effect_string:\n");
        for color in [piece::Color::Red, piece::Color::Black] {
            let effects = self.get_effects_from_color(color);
            let count = effects.len();
            for effect in effects {
                result.push_str(&effect.to_string());
            }

            result.push_str(&format!("count: {count}\n"));
        }

        result
    }

    pub fn to_string(&self) -> String {
        let mut result = format!("bottom_color: {:?}\nkinds_to_chs:\n", self.bottom_color);
        for (index, kind) in self.kinds.iter().enumerate() {
            result.push(piece::get_ch(self.get_color(index), *kind));
            if (index + 1) % 9 == 0 {
                result.push('\n');
            }
        }

        result.push_str("\ncolor_kind_pieces:\n");
        for kind_pieces in self.color_kind_pieces {
            result.push_str(&bit_constant::get_bitatom_array_string(&kind_pieces, false));
        }

        result.push_str("\ncolor_pieces:\n");
        result.push_str(&bit_constant::get_bitatom_array_string(
            &self.color_pieces,
            false,
        ));

        result.push_str("\nall_pieces:\n");
        result.push_str(&bit_constant::get_bitatom_array_string(
            &[self.all_pieces],
            false,
        ));

        result.push_str("\nrotate_all_pieces:\n");
        result.push_str(&bit_constant::get_bitatom_array_string(
            &[self.rotate_all_pieces],
            true,
        ));

        result.push_str(&format!(
            "\nhashkey :{:016x}\nhashlock:{:016x}\n",
            self.hashkey, self.hashlock
        ));

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_board() {
        let fen_board_strings = [
            ("rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR","bottom_color: Red
kinds_to_chs:
rnbakabnr
_________
_c_____c_
p_p_p_p_p
_________
_________
P_P_P_P_P
_C_____C_
_________
RNBAKABNR

color_kind_pieces:
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- --------- --------- --------- --------- --------- --------- 
1: --------- --------- --------- --------- --------- --------- --------- 
2: --------- --------- --------- --------- --------- --------- --------- 
3: --------- --------- --------- --------- --------- --------- --------- 
4: --------- --------- --------- --------- --------- --------- --------- 
5: --------- --------- --------- --------- --------- --------- --------- 
6: --------- --------- --------- --------- --------- --------- 1-1-1-1-1 
7: --------- --------- --------- --------- --------- -1-----1- --------- 
8: --------- --------- --------- --------- --------- --------- --------- 
9: ----1---- ---1-1--- --1---1-- -1-----1- 1-------1 --------- --------- 
length: 7 	non_zero: 7
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: ----1---- ---1-1--- --1---1-- -1-----1- 1-------1 --------- --------- 
1: --------- --------- --------- --------- --------- --------- --------- 
2: --------- --------- --------- --------- --------- -1-----1- --------- 
3: --------- --------- --------- --------- --------- --------- 1-1-1-1-1 
4: --------- --------- --------- --------- --------- --------- --------- 
5: --------- --------- --------- --------- --------- --------- --------- 
6: --------- --------- --------- --------- --------- --------- --------- 
7: --------- --------- --------- --------- --------- --------- --------- 
8: --------- --------- --------- --------- --------- --------- --------- 
9: --------- --------- --------- --------- --------- --------- --------- 
length: 7 	non_zero: 7

color_pieces:
   ABCDEFGHI ABCDEFGHI 
0: --------- 111111111 
1: --------- --------- 
2: --------- -1-----1- 
3: --------- 1-1-1-1-1 
4: --------- --------- 
5: --------- --------- 
6: 1-1-1-1-1 --------- 
7: -1-----1- --------- 
8: --------- --------- 
9: 111111111 --------- 
length: 2 	non_zero: 2

all_pieces:
   ABCDEFGHI 
0: 111111111 
1: --------- 
2: -1-----1- 
3: 1-1-1-1-1 
4: --------- 
5: --------- 
6: 1-1-1-1-1 
7: -1-----1- 
8: --------- 
9: 111111111 
length: 1 	non_zero: 1

rotate_all_pieces:
   ABCDEFGHIJ 
0: 1--1--1--1 
1: 1-1----1-1 
2: 1--1--1--1 
3: 1--------1 
4: 1--1--1--1 
5: 1--------1 
6: 1--1--1--1 
7: 1-1----1-1 
8: 1--1--1--1 
length: 1 	non_zero: 1

hashkey :a7723f5bf923d819
hashlock:5a278e0f64c5677a

moves_string:
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- --------- --------- --------- --------- -1------- -------1- --------- --------- 
1: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
2: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
3: --------- --------- --------- --------- --------- -1------- -------1- --------- --------- 
4: --------- --------- --------- --------- --------- -1------- -------1- --------- --------- 
5: 1-------- --1------ ----1---- ------1-- --------1 -1------- -------1- --------- --------- 
6: --------- --------- --------- --------- --------- -1------- -------1- --------- --------- 
7: --------- --------- --------- --------- --------- 1-11111-- --11111-1 1-------- 1-1------ 
8: --------- --------- --------- --------- --------- -1------- -------1- 1-------- --------- 
9: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- --------- --------- --------- --------- --------- --------- 
1: --------- --------- --------- --------- --------- --------- --------- 
2: --------- --------- --------- --------- --------- --------- --------- 
3: --------- --------- --------- --------- --------- --------- --------- 
4: --------- --------- --------- --------- --------- --------- --------- 
5: --------- --------- --------- --------- --------- --------- --------- 
6: --------- --------- --------- --------- --------- --------- --------- 
7: 1---1---- --------- --------- --------- ----1---1 ------1-1 --------1 
8: --------- ----1---- ----1---- ----1---- --------- --------- --------1 
9: --------- --------- --------- --------- --------- --------- --------- 
length: 16 	non_zero: 16
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
1: 1-------- --------- --------- ----1---- ----1---- ----1---- --------- --------- --------1 
2: 1-------- 1-1------ 1---1---- --------- --------- --------- ----1---1 ------1-1 --------1 
3: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
4: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
5: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
6: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
7: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
8: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
9: --------- --------- --------- --------- --------- --------- --------- --------- --------- 
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- --------- --------- --------- --------- --------- --------- 
1: -1------- -------1- --------- --------- --------- --------- --------- 
2: 1-11111-- --11111-1 --------- --------- --------- --------- --------- 
3: -1------- -------1- --------- --------- --------- --------- --------- 
4: -1------- -------1- 1-------- --1------ ----1---- ------1-- --------1 
5: -1------- -------1- --------- --------- --------- --------- --------- 
6: -1------- -------1- --------- --------- --------- --------- --------- 
7: --------- --------- --------- --------- --------- --------- --------- 
8: --------- --------- --------- --------- --------- --------- --------- 
9: -1------- -------1- --------- --------- --------- --------- --------- 
length: 16 	non_zero: 16

effect_string:
[6,0] => (5,0)-0-0 【1】
[6,2] => (5,2)-0-0 【1】
[6,4] => (5,4)-0-0 【1】
[6,6] => (5,6)-0-0 【1】
[6,8] => (5,8)-0-0 【1】
[7,1] => (0,1)-0-0 (3,1)-0-0 (4,1)-0-0 (5,1)-0-0 (6,1)-0-0 (7,0)-0-0 (7,2)-0-0 (7,3)-0-0 (7,4)-0-0 (7,5)-0-0 (7,6)-0-0 (8,1)-0-0 【12】
[7,7] => (0,7)-0-0 (3,7)-0-0 (4,7)-0-0 (5,7)-0-0 (6,7)-0-0 (7,2)-0-0 (7,3)-0-0 (7,4)-0-0 (7,5)-0-0 (7,6)-0-0 (7,8)-0-0 (8,7)-0-0 【12】
[9,0] => (7,0)-0-0 (8,0)-0-0 【2】
[9,1] => (7,0)-0-0 (7,2)-0-0 【2】
[9,2] => (7,0)-0-0 (7,4)-0-0 【2】
[9,3] => (8,4)-0-0 【1】
[9,4] => (8,4)-0-0 【1】
[9,5] => (8,4)-0-0 【1】
[9,6] => (7,4)-0-0 (7,8)-0-0 【2】
[9,7] => (7,6)-0-0 (7,8)-0-0 【2】
[9,8] => (7,8)-0-0 (8,8)-0-0 【2】
count: 16
[0,0] => (1,0)-0-0 (2,0)-0-0 【2】
[0,1] => (2,0)-0-0 (2,2)-0-0 【2】
[0,2] => (2,0)-0-0 (2,4)-0-0 【2】
[0,3] => (1,4)-0-0 【1】
[0,4] => (1,4)-0-0 【1】
[0,5] => (1,4)-0-0 【1】
[0,6] => (2,4)-0-0 (2,8)-0-0 【2】
[0,7] => (2,6)-0-0 (2,8)-0-0 【2】
[0,8] => (1,8)-0-0 (2,8)-0-0 【2】
[2,1] => (1,1)-0-0 (2,0)-0-0 (2,2)-0-0 (2,3)-0-0 (2,4)-0-0 (2,5)-0-0 (2,6)-0-0 (3,1)-0-0 (4,1)-0-0 (5,1)-0-0 (6,1)-0-0 (9,1)-0-0 【12】
[2,7] => (1,7)-0-0 (2,2)-0-0 (2,3)-0-0 (2,4)-0-0 (2,5)-0-0 (2,6)-0-0 (2,8)-0-0 (3,7)-0-0 (4,7)-0-0 (5,7)-0-0 (6,7)-0-0 (9,7)-0-0 【12】
[3,0] => (4,0)-0-0 【1】
[3,2] => (4,2)-0-0 【1】
[3,4] => (4,4)-0-0 【1】
[3,6] => (4,6)-0-0 【1】
[3,8] => (4,8)-0-0 【1】
count: 16
"),
            ("5a3/4ak2r/6R2/8p/9/9/9/B4N2B/4K4/3c5","bottom_color: Red
kinds_to_chs:
_____a___
____ak__r
______R__
________p
_________
_________
_________
B____N__B
____K____
___c_____

color_kind_pieces:
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- --------- --------- --------- --------- --------- --------- 
1: --------- --------- --------- --------- --------- --------- --------- 
2: --------- --------- --------- --------- ------1-- --------- --------- 
3: --------- --------- --------- --------- --------- --------- --------- 
4: --------- --------- --------- --------- --------- --------- --------- 
5: --------- --------- --------- --------- --------- --------- --------- 
6: --------- --------- --------- --------- --------- --------- --------- 
7: --------- --------- 1-------1 -----1--- --------- --------- --------- 
8: ----1---- --------- --------- --------- --------- --------- --------- 
9: --------- --------- --------- --------- --------- --------- --------- 
length: 7 	non_zero: 4
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- -----1--- --------- --------- --------- --------- --------- 
1: -----1--- ----1---- --------- --------- --------1 --------- --------- 
2: --------- --------- --------- --------- --------- --------- --------- 
3: --------- --------- --------- --------- --------- --------- --------1 
4: --------- --------- --------- --------- --------- --------- --------- 
5: --------- --------- --------- --------- --------- --------- --------- 
6: --------- --------- --------- --------- --------- --------- --------- 
7: --------- --------- --------- --------- --------- --------- --------- 
8: --------- --------- --------- --------- --------- --------- --------- 
9: --------- --------- --------- --------- --------- ---1----- --------- 
length: 7 	non_zero: 5

color_pieces:
   ABCDEFGHI ABCDEFGHI 
0: --------- -----1--- 
1: --------- ----11--1 
2: ------1-- --------- 
3: --------- --------1 
4: --------- --------- 
5: --------- --------- 
6: --------- --------- 
7: 1----1--1 --------- 
8: ----1---- --------- 
9: --------- ---1----- 
length: 2 	non_zero: 2

all_pieces:
   ABCDEFGHI 
0: -----1--- 
1: ----11--1 
2: ------1-- 
3: --------1 
4: --------- 
5: --------- 
6: --------- 
7: 1----1--1 
8: ----1---- 
9: ---1----- 
length: 1 	non_zero: 1

rotate_all_pieces:
   ABCDEFGHIJ 
0: -------1-- 
1: ---------- 
2: ---------- 
3: ---------1 
4: -1------1- 
5: 11-----1-- 
6: --1------- 
7: ---------- 
8: -1-1---1-- 
length: 1 	non_zero: 1

hashkey :ca2f328f172f2d56
hashlock:61fb68a5da82cf13

moves_string:
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: ------1-- --------- --------- --------- --------- 
1: ------1-- --------- --------- --------- --------- 
2: 111111-11 --------- --------- --------- --------- 
3: ------1-- --------- --------- --------- --------- 
4: ------1-- --------- --------- --------- --------- 
5: ------1-- --1------ ----1-1-- ------1-- --------- 
6: ------1-- --------- ---1---1- --------- --------- 
7: ------1-- --------- --------- --------- ----1---- 
8: ------1-- --------- ---1---1- --------- ---1-1--- 
9: ------1-- --1------ ----1-1-- ------1-- ----1---- 
length: 5 	non_zero: 5
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- ---1----- --------- --------1 --------- ---1----- 
1: --------- --------- --------- ------11- --------- ---1----- 
2: --------- ---1-1--- -----1--- --------1 --------- ---1----- 
3: --------- --------- --------- --------- --------- ---1----- 
4: --------- --------- --------- --------- --------1 ---1----- 
5: --------- --------- --------- --------- --------- ---1----- 
6: --------- --------- --------- --------- --------- ---1----- 
7: --------- --------- --------- --------- --------- ---1----- 
8: --------- --------- --------- --------- --------- ---1----- 
9: --------- --------- --------- --------- --------- 111-11111 
length: 6 	non_zero: 5

effect_string:
[2,6] => (0,6)-0-0 (1,6)-0-0 (2,0)-0-0 (2,1)-0-0 (2,2)-0-0 (2,3)-0-0 (2,4)-0-0 (2,5)-0-0 (2,7)-0-0 (2,8)-0-0 (3,6)-0-0 (4,6)-0-0 (5,6)-0-0 (6,6)-0-0 (7,6)-0-0 (8,6)-0-0 (9,6)-0-0 【17】
[7,0] => (5,2)-0-0 (9,2)-0-0 【2】
[7,5] => (5,4)-0-0 (5,6)-0-0 (6,3)-0-0 (6,7)-0-0 (8,3)-0-0 (8,7)-0-0 (9,4)-0-0 (9,6)-0-0 【8】
[7,8] => (5,6)-0-0 (9,6)-0-0 【2】
[8,4] => (7,4)-0-0 (8,3)-0-0 (8,5)-0-0 (9,4)-0-0 【4】
count: 5
[0,5] => 【0】
[1,4] => (0,3)-0-0 (2,3)-0-0 (2,5)-0-0 【3】
[1,5] => (2,5)--1-0 【1】
[1,8] => (0,8)-0-0 (1,6)-0-0 (1,7)-0-0 (2,8)-0-0 【4】
[3,8] => (4,8)-0-0 【1】
[9,3] => (0,3)-0-0 (1,3)-0-0 (2,3)-0-0 (3,3)-0-0 (4,3)-0-0 (5,3)-0-0 (6,3)-0-0 (7,3)-0-0 (8,3)-0-0 (9,0)-0-0 (9,1)-0-0 (9,2)-0-0 (9,4)-0-0 (9,5)-0-0 (9,6)-0-0 (9,7)-0-0 (9,8)-0-0 【17】
count: 6
"),
            ("2b1kab2/4a4/4c4/9/9/3R5/9/1C7/4r4/2BK2B2","bottom_color: Red
kinds_to_chs:
__b_kab__
____a____
____c____
_________
_________
___R_____
_________
_C_______
____r____
__BK__B__

color_kind_pieces:
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- --------- --------- --------- --------- --------- --------- 
1: --------- --------- --------- --------- --------- --------- --------- 
2: --------- --------- --------- --------- --------- --------- --------- 
3: --------- --------- --------- --------- --------- --------- --------- 
4: --------- --------- --------- --------- --------- --------- --------- 
5: --------- --------- --------- --------- ---1----- --------- --------- 
6: --------- --------- --------- --------- --------- --------- --------- 
7: --------- --------- --------- --------- --------- -1------- --------- 
8: --------- --------- --------- --------- --------- --------- --------- 
9: ---1----- --------- --1---1-- --------- --------- --------- --------- 
length: 7 	non_zero: 4
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: ----1---- -----1--- --1---1-- --------- --------- --------- --------- 
1: --------- ----1---- --------- --------- --------- --------- --------- 
2: --------- --------- --------- --------- --------- ----1---- --------- 
3: --------- --------- --------- --------- --------- --------- --------- 
4: --------- --------- --------- --------- --------- --------- --------- 
5: --------- --------- --------- --------- --------- --------- --------- 
6: --------- --------- --------- --------- --------- --------- --------- 
7: --------- --------- --------- --------- --------- --------- --------- 
8: --------- --------- --------- --------- ----1---- --------- --------- 
9: --------- --------- --------- --------- --------- --------- --------- 
length: 7 	non_zero: 5

color_pieces:
   ABCDEFGHI ABCDEFGHI 
0: --------- --1-111-- 
1: --------- ----1---- 
2: --------- ----1---- 
3: --------- --------- 
4: --------- --------- 
5: ---1----- --------- 
6: --------- --------- 
7: -1------- --------- 
8: --------- ----1---- 
9: --11--1-- --------- 
length: 2 	non_zero: 2

all_pieces:
   ABCDEFGHI 
0: --1-111-- 
1: ----1---- 
2: ----1---- 
3: --------- 
4: --------- 
5: ---1----- 
6: --------- 
7: -1------- 
8: ----1---- 
9: --11--1-- 
length: 1 	non_zero: 1

rotate_all_pieces:
   ABCDEFGHIJ 
0: ---------- 
1: -------1-- 
2: 1--------1 
3: -----1---1 
4: 111-----1- 
5: 1--------- 
6: 1--------1 
7: ---------- 
8: ---------- 
length: 1 	non_zero: 1

hashkey :76ac33af29a8120e
hashlock:e6fe0215a38b5352

moves_string:
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: ---1----- -1------- --------- --------- --------- 
1: ---1----- -1------- --------- --------- --------- 
2: ---1----- -1------- --------- --------- --------- 
3: ---1----- -1------- --------- --------- --------- 
4: ---1----- -1------- --------- --------- --------- 
5: 111-11111 -1------- --------- --------- --------- 
6: ---1----- -1------- --------- --------- --------- 
7: ---1----- 1-1111111 1---1---- --------- ----1---1 
8: ---1----- -1------- --------- ---1----- --------- 
9: --------- -1------- --------- ----1---- --------- 
length: 5 	non_zero: 5
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- ---1----- --------- --------- ---1----- --------- --------- 
1: --------- --------- --------- --------- --------- --------- --------- 
2: 1-------- --------- --------- --------1 ---1-1--- 1111-1111 --------- 
3: --------- --------- --------- --------- --------- ----1---- ----1---- 
4: --------- --------- --------- --------- --------- ----1---- ----1---- 
5: --------- --------- --------- --------- --------- ----1---- ----1---- 
6: --------- --------- --------- --------- --------- ----1---- ----1---- 
7: --------- --------- --------- --------- --------- ----1---- ----1---- 
8: --------- --------- --------- --------- --------- --------- 1111-1111 
9: --------- --------- --------- --------- --------- --------- ----1---- 
length: 7 	non_zero: 6

effect_string:
[5,3] => (0,3)-0-0 (1,3)-0-0 (2,3)-0-0 (3,3)-0-0 (4,3)-0-0 (5,0)-0-0 (5,1)-0-0 (5,2)-0-0 (5,4)-0-0 (5,5)-0-0 (5,6)-0-0 (5,7)-0-0 (5,8)-0-0 (6,3)-0-0 (7,3)-0-0 (8,3)-0-0 【16】
[7,1] => (0,1)-0-0 (1,1)-0-0 (2,1)-0-0 (3,1)-0-0 (4,1)-0-0 (5,1)-0-0 (6,1)-0-0 (7,0)-0-0 (7,2)-0-0 (7,3)-0-0 (7,4)-0-0 (7,5)-0-0 (7,6)-0-0 (7,7)-0-0 (7,8)-0-0 (8,1)-0-0 (9,1)-0-0 【17】
[9,2] => (7,0)-0-0 (7,4)-0-0 【2】
[9,3] => (8,3)--1-0 (9,4)--1-0 【2】
[9,6] => (7,4)-0-0 (7,8)-0-0 【2】
count: 5
[0,2] => (2,0)-0-0 【1】
[0,4] => (0,3)--1-0 【1】
[0,5] => 【0】
[0,6] => (2,8)-0-0 【1】
[1,4] => (0,3)-0-0 (2,3)-0-0 (2,5)-0-0 【3】
[2,4] => (2,0)-0-0 (2,1)-0-0 (2,2)-0-0 (2,3)-0-0 (2,5)-0-0 (2,6)-0-0 (2,7)-0-0 (2,8)-0-0 (3,4)-0-0 (4,4)-0-0 (5,4)-0-0 (6,4)-0-0 (7,4)-0-0 【13】
[8,4] => (3,4)-0-0 (4,4)-0-0 (5,4)-0-0 (6,4)-0-0 (7,4)-0-0 (8,0)-0-0 (8,1)-0-0 (8,2)-0-0 (8,3)-0-0 (8,5)-0-0 (8,6)-0-0 (8,7)-0-0 (8,8)-0-0 (9,4)-0-0 【14】
count: 7
"),
            ("4kab2/4a4/4b4/3N5/9/4N4/4n4/4B4/4A4/3AK1B2","bottom_color: Red
kinds_to_chs:
____kab__
____a____
____b____
___N_____
_________
____N____
____n____
____B____
____A____
___AK_B__

color_kind_pieces:
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- --------- --------- --------- --------- --------- --------- 
1: --------- --------- --------- --------- --------- --------- --------- 
2: --------- --------- --------- --------- --------- --------- --------- 
3: --------- --------- --------- ---1----- --------- --------- --------- 
4: --------- --------- --------- --------- --------- --------- --------- 
5: --------- --------- --------- ----1---- --------- --------- --------- 
6: --------- --------- --------- --------- --------- --------- --------- 
7: --------- --------- ----1---- --------- --------- --------- --------- 
8: --------- ----1---- --------- --------- --------- --------- --------- 
9: ----1---- ---1----- ------1-- --------- --------- --------- --------- 
length: 7 	non_zero: 4
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: ----1---- -----1--- ------1-- --------- --------- --------- --------- 
1: --------- ----1---- --------- --------- --------- --------- --------- 
2: --------- --------- ----1---- --------- --------- --------- --------- 
3: --------- --------- --------- --------- --------- --------- --------- 
4: --------- --------- --------- --------- --------- --------- --------- 
5: --------- --------- --------- --------- --------- --------- --------- 
6: --------- --------- --------- ----1---- --------- --------- --------- 
7: --------- --------- --------- --------- --------- --------- --------- 
8: --------- --------- --------- --------- --------- --------- --------- 
9: --------- --------- --------- --------- --------- --------- --------- 
length: 7 	non_zero: 4

color_pieces:
   ABCDEFGHI ABCDEFGHI 
0: --------- ----111-- 
1: --------- ----1---- 
2: --------- ----1---- 
3: ---1----- --------- 
4: --------- --------- 
5: ----1---- --------- 
6: --------- ----1---- 
7: ----1---- --------- 
8: ----1---- --------- 
9: ---11-1-- --------- 
length: 2 	non_zero: 2

all_pieces:
   ABCDEFGHI 
0: ----111-- 
1: ----1---- 
2: ----1---- 
3: ---1----- 
4: --------- 
5: ----1---- 
6: ----1---- 
7: ----1---- 
8: ----1---- 
9: ---11-1-- 
length: 1 	non_zero: 1

rotate_all_pieces:
   ABCDEFGHIJ 
0: ---------- 
1: ---------- 
2: ---------- 
3: ---1-----1 
4: 111--11111 
5: 1--------- 
6: 1--------1 
7: ---------- 
8: ---------- 
length: 1 	non_zero: 1

hashkey :67384276a2d3addf
hashlock:e51b108b9aa27e49

moves_string:
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: --------- --------- --------- --------- --------- --------- --------- 
1: --1-1---- --------- --------- --------- --------- --------- --------- 
2: -1---1--- --------- --------- --------- --------- --------- --------- 
3: --------- -----1--- --------- --------- --------- --------- --------- 
4: -1---1--- --1---1-- --------- --------- --------- --------- --------- 
5: --1------ --------- --1---1-- --------- --------- --------- --------- 
6: --------- --1---1-- --------- --------- --------- --------- --------- 
7: --------- --------- --------- ---1-1--- --------- --------- --------1 
8: --------- --------- --------- --------- --------- --------- --------- 
9: --------- --------- --1------ -----1--- --------- -----1--- --------- 
length: 7 	non_zero: 6
   ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI ABCDEFGHI 
0: ---1----- --------- --------- ---1----- --1------ --------- 
1: --------- --------- --------- --------- --------- --------- 
2: --------- --------- --------1 ---1-1--- --------- --------- 
3: --------- --------- --------- --------- --------- --------- 
4: --------- --------- --------- --------- ------1-- --------- 
5: --------- --------- --------- --------- --------- --1---1-- 
6: --------- --------- --------- --------- --------- --------- 
7: --------- --------- --------- --------- --------- --1---1-- 
8: --------- --------- --------- --------- --------- --------- 
9: --------- --------- --------- --------- --------- --------- 
length: 6 	non_zero: 5

effect_string:
[3,3] => (1,2)-0-0 (1,4)-0-0 (2,1)-0-0 (2,5)-0-0 (4,1)-0-0 (4,5)-0-0 (5,2)-0-0 【7】
[5,4] => (3,5)-0-0 (4,2)-0-0 (4,6)-0-0 (6,2)-0-0 (6,6)-0-0 【5】
[7,4] => (5,2)-0-0 (5,6)-0-0 (9,2)-0-0 【3】
[8,4] => (7,3)-0-0 (7,5)-0-0 (9,5)-0-0 【3】
[9,3] => 【0】
[9,4] => (9,5)-0-0 【1】
[9,6] => (7,8)-0-0 【1】
count: 7
[0,4] => (0,3)-0-0 【1】
[0,5] => 【0】
[0,6] => (2,8)-0-0 【1】
[1,4] => (0,3)-0-0 (2,3)-0-0 (2,5)-0-0 【3】
[2,4] => (0,2)-0-0 (4,6)-0-0 【2】
[6,4] => (5,2)-0-0 (5,6)-0-0 (7,2)-0-0 (7,6)-0-0 【4】
count: 6
"),
        ];

        for (fen, board_string) in fen_board_strings {
            let mut bit_board = BitBoard::new(&board::fen_to_pieces(fen));
            let mut result = bit_board.to_string();

            result.push('\n');
            result.push_str(&bit_board.to_moves_string());

            result.push('\n');
            result.push_str(&bit_board.to_effects_string());

            assert_eq!(board_string, result);

            // let name = fen.split_at(3).0;
            // std::fs::write(format!("tests/{name}.txt"), result).expect("Write Err.");
            // dbg!(bit_board);
        }
    }
}
