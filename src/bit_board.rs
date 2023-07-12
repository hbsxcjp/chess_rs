#![allow(dead_code)]
#![allow(unused_imports)]

use crate::bit_constant;
use crate::board;
use crate::coord::{
    self, COLCOUNT, COLSTATECOUNT, LEGSTATECOUNT, ROWCOUNT, ROWSTATECOUNT, SEATCOUNT, SIDECOUNT,
};
use crate::evaluation;
use crate::piece::{self, COLORCOUNT, KINDCOUNT};

type OperateEvaluation = fn(
    &BitBoard,
    aspect_evaluation: &mut evaluation::AspectEvaluation,
    from_to_index: (usize, usize),
    eat_kind: piece::Kind,
);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BitBoard {
    bottom_color: piece::Color,
    kinds: [piece::Kind; SEATCOUNT],

    // 计算中间存储数据(基本局面改动时更新)
    color_kind_pieces: [[bit_constant::BitAtom; KINDCOUNT]; COLORCOUNT],
    color_pieces: [bit_constant::BitAtom; COLORCOUNT],
    all_pieces: bit_constant::BitAtom,
    rotate_all_pieces: bit_constant::BitAtom,

    // 哈希局面数据
    key: u64,
    lock: u64,
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

            key: 0,
            lock: 0,
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

                bit_board.key ^= bit_constant::ZOBRISTKEY[color_i][kind_i][index];
                bit_board.lock ^= bit_constant::ZOBRISTLOCK[color_i][kind_i][index];
            }
        }

        bit_board
    }

    pub fn get_color(&self, index: usize) -> Option<piece::Color> {
        let index_mask = bit_constant::MASK[index];
        if self.color_pieces[piece::Color::Red as usize] & index_mask != 0 {
            Some(piece::Color::Red)
        } else if self.color_pieces[piece::Color::Black as usize] & index_mask != 0 {
            Some(piece::Color::Black)
        } else {
            None
        }
    }

    fn get_move_from_index(&self, index: usize) -> bit_constant::BitAtom {
        let color = self.get_color(index).unwrap();
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

    fn is_killed(&self, color: piece::Color) -> bool {
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

    fn is_failed(&self, color: piece::Color) -> bool {
        self.get_move_from_color(color) == 0
    }

    pub fn do_move(&mut self, from_index: usize, to_index: usize) -> piece::Kind {
        self.operate_move(from_index, to_index, false, piece::Kind::NoKind)
    }

    pub fn undo_move(
        &mut self,
        from_index: usize,
        to_index: usize,
        eat_kind: piece::Kind,
    ) -> piece::Kind {
        self.operate_move(from_index, to_index, true, eat_kind)
    }

    fn operate_move(
        &mut self,
        from_index: usize,
        to_index: usize,
        is_undo: bool,
        mut eat_kind: piece::Kind,
    ) -> piece::Kind {
        let move_from_index = if is_undo { to_index } else { from_index };
        let move_to_index = if is_undo { from_index } else { to_index };
        let from_color_i = self.get_color(move_from_index).unwrap() as usize;
        let from_kind = self.kinds[move_from_index];
        let from_kind_i = from_kind as usize;
        let move_bitatom = bit_constant::MASK[from_index] | bit_constant::MASK[to_index];

        // 清除原位置，置位新位置
        self.kinds[move_from_index] = eat_kind;
        if !is_undo {
            eat_kind = self.kinds[to_index];
        }
        self.kinds[move_to_index] = from_kind;

        // 设置颜色种类位棋盘
        self.color_kind_pieces[from_color_i][from_kind_i] ^= move_bitatom;
        self.color_pieces[from_color_i] ^= move_bitatom;

        self.key ^= bit_constant::ZOBRISTKEY[from_color_i][from_kind_i][from_index]
            ^ bit_constant::ZOBRISTKEY[from_color_i][from_kind_i][to_index];
        self.lock ^= bit_constant::ZOBRISTLOCK[from_color_i][from_kind_i][from_index]
            ^ bit_constant::ZOBRISTLOCK[from_color_i][from_kind_i][to_index];

        match eat_kind {
            piece::Kind::NoKind => {
                self.all_pieces ^= move_bitatom;
                self.rotate_all_pieces ^=
                    bit_constant::ROTATEMASK[from_index] | bit_constant::ROTATEMASK[to_index];
            }
            _ => {
                let to_color_i = if from_color_i == 0 { 1 } else { 0 };
                let eat_kind_i = eat_kind as usize;
                let eat_bitatom = bit_constant::MASK[to_index];

                // 设置颜色种类位棋盘
                self.color_kind_pieces[to_color_i][eat_kind_i] ^= eat_bitatom;
                self.color_pieces[to_color_i] ^= eat_bitatom;

                self.key ^= bit_constant::ZOBRISTKEY[to_color_i][eat_kind_i][to_index];
                self.lock ^= bit_constant::ZOBRISTLOCK[to_color_i][eat_kind_i][to_index];

                self.all_pieces ^= bit_constant::MASK[from_index];
                self.rotate_all_pieces ^= bit_constant::ROTATEMASK[from_index];
            }
        }

        eat_kind
    }

    fn add_evaluation_is_killed(
        &self,
        aspect_evaluation: &mut evaluation::AspectEvaluation,
        from_to_index: (usize, usize),
        eat_kind: piece::Kind,
    ) {
        let (from_index, to_index) = from_to_index;
        // 如是对方将帅的位置则直接可走，不用判断是否被将军（如加以判断，则会直接走棋吃将帅）；棋子已走，取终点位置颜色
        let is_killed =
            eat_kind != piece::Kind::King && self.is_killed(self.get_color(to_index).unwrap());
        let count = if is_killed { 0 } else { 1 };
        // 扩展，增加其他功能

        aspect_evaluation.insert_evaluation(
            from_index,
            to_index,
            evaluation::Evaluation::new(is_killed, eat_kind, count),
        );
    }

    // 执行某一着不回退
    fn operate_evaluation_by_do_move(
        &mut self,
        aspect_evaluation: &mut evaluation::AspectEvaluation,
        from_to_index: (usize, usize),
        operate_evaluation: OperateEvaluation,
    ) {
        let (from_index, to_index) = from_to_index;
        let eat_kind = self.do_move(from_index, to_index);
        operate_evaluation(self, aspect_evaluation, from_to_index, eat_kind);
    }

    // 执行某一着后回退（保持原局面不变）
    fn operate_evaluation_by_do_move_undo(
        &mut self,
        aspect_evaluation: &mut evaluation::AspectEvaluation,
        from_to_index: (usize, usize),
        operate_evaluation: OperateEvaluation,
    ) {
        let (from_index, to_index) = from_to_index;
        let eat_kind = self.do_move(from_index, to_index);
        operate_evaluation(self, aspect_evaluation, from_to_index, eat_kind);

        self.undo_move(from_index, to_index, eat_kind);
    }

    fn get_aspect_evaluation(
        &mut self,
        from_to_index: (usize, usize),
    ) -> evaluation::AspectEvaluation {
        let mut aspect_evaluation = evaluation::AspectEvaluation::new();
        self.operate_evaluation_by_do_move(
            &mut aspect_evaluation,
            from_to_index,
            Self::add_evaluation_is_killed,
        );

        aspect_evaluation
    }

    fn get_aspect_evaluation_from_index(
        &mut self,
        from_index: usize,
    ) -> evaluation::AspectEvaluation {
        let mut aspect_evaluation = evaluation::AspectEvaluation::from(from_index);
        for to_index in bit_constant::get_indexs_from_bitatom(self.get_move_from_index(from_index))
        {
            self.operate_evaluation_by_do_move_undo(
                &mut aspect_evaluation,
                (from_index, to_index),
                Self::add_evaluation_is_killed,
            );
        }

        aspect_evaluation
    }

    fn get_aspect_evaluation_from_bitatom(
        &mut self,
        bit_atom: bit_constant::BitAtom,
    ) -> evaluation::AspectEvaluation {
        let aspect_evaluation = evaluation::AspectEvaluation::new();
        for from_index in bit_constant::get_indexs_from_bitatom(bit_atom) {
            aspect_evaluation.append(self.get_aspect_evaluation_from_index(from_index));
        }

        aspect_evaluation
    }

    // kind == piece::Kind::NoKind，取全部种类棋子
    fn get_aspect_evaluation_from_color_kind(
        &mut self,
        color: piece::Color,
        kind: piece::Kind,
    ) -> evaluation::AspectEvaluation {
        self.get_aspect_evaluation_from_bitatom(match kind {
            piece::Kind::NoKind => self.color_pieces[color as usize],
            _ => self.color_kind_pieces[color as usize][kind as usize],
        })
    }

    pub fn get_aspect_evaluation_from_color(
        &mut self,
        color: piece::Color,
    ) -> evaluation::AspectEvaluation {
        self.get_aspect_evaluation_from_color_kind(color, piece::Kind::NoKind)
    }

    pub fn get_key(&self, color: piece::Color) -> u64 {
        self.key ^ bit_constant::COLORZOBRISTKEY[color as usize]
    }

    pub fn get_lock(&self, color: piece::Color) -> u64 {
        self.lock ^ bit_constant::COLORZOBRISTLOCK[color as usize]
    }

    fn set_zorbist_evaluation0(
        &self,
        color: piece::Color,
        zorbist_aspect_evaluation: &mut evaluation::ZorbistAspectEvaluation,
        aspect_evaluation: evaluation::AspectEvaluation,
    ) {
        zorbist_aspect_evaluation.insert(
            self.get_key(color),
            self.get_lock(color),
            aspect_evaluation,
        );
    }

    pub fn get_zorbist_evaluation_from_color(
        &mut self,
        color: piece::Color,
    ) -> evaluation::ZorbistAspectEvaluation {
        let mut zorbist_aspect_evaluation = evaluation::ZorbistAspectEvaluation::new();
        let aspect_evaluation = self.get_aspect_evaluation_from_color(color);
        self.set_zorbist_evaluation0(color, &mut zorbist_aspect_evaluation, aspect_evaluation);

        zorbist_aspect_evaluation
    }

    pub fn get_zorbist_evaluation_from_to_index(
        &mut self,
        color: piece::Color,
        from_to_index: (usize, usize),
    ) -> evaluation::ZorbistAspectEvaluation {
        let mut zorbist_aspect_evaluation = evaluation::ZorbistAspectEvaluation::new();
        let aspect_evaluation = self.get_aspect_evaluation(from_to_index);
        self.set_zorbist_evaluation0(color, &mut zorbist_aspect_evaluation, aspect_evaluation);

        zorbist_aspect_evaluation
    }

    pub fn to_string(&mut self) -> String {
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
            self.key, self.lock
        ));

        // 可变借用
        fn to_moves_string(bit_board: &mut BitBoard) -> String {
            let mut result = format!("moves_string:\n");
            for color in [piece::Color::Red, piece::Color::Black] {
                let mut moves = Vec::new();
                for index in
                    bit_constant::get_indexs_from_bitatom(bit_board.color_pieces[color as usize])
                {
                    moves.push(bit_board.get_move_from_index(index));
                }

                result.push_str(&bit_constant::get_bitatom_array_string(&moves, false));
            }

            result
        }

        result.push('\n');
        result.push_str(&to_moves_string(self));

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_board() {
        let fen_board_strings = [
            (
                "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR",
                "bottom_color: Red
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
",
            ),
            (
                "5a3/4ak2r/6R2/8p/9/9/9/B4N2B/4K4/3c5",
                "bottom_color: Red
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
",
            ),
            (
                "2b1kab2/4a4/4c4/9/9/3R5/9/1C7/4r4/2BK2B2",
                "bottom_color: Red
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
",
            ),
            (
                "4kab2/4a4/4b4/3N5/9/4N4/4n4/4B4/4A4/3AK1B2",
                "bottom_color: Red
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
",
            ),
        ];

        for (fen, board_string) in fen_board_strings {
            let mut bit_board = BitBoard::new(&board::fen_to_pieces(fen));
            let mut result = bit_board.to_string();

            assert_eq!(board_string, result);

            // // 可变借用
            fn to_aspect_evaluation_string(bit_board: &mut BitBoard) -> String {
                let mut result = format!("aspect_evaluation_string:\n");
                for color in [piece::Color::Red, piece::Color::Black] {
                    let zorbist_aspect_evaluation =
                        bit_board.get_zorbist_evaluation_from_color(color);
                    result.push_str(&zorbist_aspect_evaluation.to_string());
                }

                result
            }

            result.push('\n');
            result.push_str(&to_aspect_evaluation_string(&mut bit_board));

            let name = fen.split_at(3).0;
            std::fs::write(format!("tests/output/bit_board_{name}.txt"), result)
                .expect("Write Err.");
            // dbg!(bit_board);
        }
    }
}
