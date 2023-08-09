#![allow(dead_code)]
#![allow(unused_imports)]

use crate::amove;
use crate::bit_constant;
use crate::board;
use crate::coord::{
    self, COLCOUNT, COLSTATECOUNT, LEGSTATECOUNT, ROWCOUNT, ROWSTATECOUNT, SEATCOUNT, SIDECOUNT,
};
use crate::evaluation;
use crate::manual;
use crate::manual_move;
use crate::models;
use crate::piece::{self, COLORCOUNT, KINDCOUNT};
use std::rc::Rc;

type GetEvaluation =
    fn(&BitBoard, from_to_index: (usize, usize), eat_kind: piece::Kind) -> evaluation::Evaluation;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BitBoard {
    bottom_color: piece::Color,

    // 计算中间存储数据(基本局面改动时更新)
    bit_pieces: [[bit_constant::BitAtom; KINDCOUNT]; COLORCOUNT],

    // 哈希局面数据
    key: u64,
    lock: u64,
}

impl BitBoard {
    pub fn new(pieces: &board::Pieces) -> BitBoard {
        let mut bit_board: BitBoard = BitBoard {
            bottom_color: board::get_bottom_color(pieces),
            bit_pieces: [[0; KINDCOUNT]; COLORCOUNT],

            key: 0,
            lock: 0,
        };

        for (index, piece) in pieces.iter().enumerate() {
            if let piece::Piece::Some(color, kind) = piece {
                let color_i = *color as usize;
                let kind_i = *kind as usize;

                bit_board.bit_pieces[color_i][kind_i] |= bit_constant::MASK[index];

                bit_board.key ^= bit_constant::ZOBRISTKEY[color_i][kind_i][index];
                bit_board.lock ^= bit_constant::ZOBRISTLOCK[color_i][kind_i][index];
            }
        }

        bit_board
    }

    fn get_color(&self, index: usize) -> Option<piece::Color> {
        let index_mask = bit_constant::MASK[index];
        if self.color_pieces(piece::Color::Red) & index_mask != 0 {
            Some(piece::Color::Red)
        } else if self.color_pieces(piece::Color::Black) & index_mask != 0 {
            Some(piece::Color::Black)
        } else {
            None
        }
    }

    pub fn get_key_lock(&self, color: piece::Color) -> (u64, u64) {
        (self.get_key(color), self.get_lock(color))
    }

    fn get_kind(&self, index: usize) -> piece::Kind {
        let index_mask = bit_constant::MASK[index];
        for kind in [
            piece::Kind::Rook,
            piece::Kind::Knight,
            piece::Kind::Cannon,
            piece::Kind::Pawn,
            piece::Kind::King,
            piece::Kind::Advisor,
            piece::Kind::Bishop,
        ] {
            let kind_piece = self.bit_pieces[piece::Color::Red as usize][kind as usize]
                | self.bit_pieces[piece::Color::Black as usize][kind as usize];
            if kind_piece & index_mask != 0 {
                return kind;
            }
        }

        piece::Kind::NoKind
    }

    fn color_pieces(&self, color: piece::Color) -> bit_constant::BitAtom {
        let mut result = 0;
        for kind_piece in self.bit_pieces[color as usize] {
            result |= kind_piece;
        }

        result
    }

    fn all_pieces(&self) -> bit_constant::BitAtom {
        self.color_pieces(piece::Color::Red) | self.color_pieces(piece::Color::Black)
    }

    fn rotate_all_pieces(&self) -> bit_constant::BitAtom {
        let mut result = 0;
        for index in bit_constant::get_indexs_from_bitatom(self.all_pieces()) {
            result |= bit_constant::ROTATEMASK[index];
        }

        result
    }

    fn get_move_from_index(&self, index: usize) -> bit_constant::BitAtom {
        let color = self.get_color(index).unwrap();
        let kind = self.get_kind(index);
        let result = match kind {
            piece::Kind::King => bit_constant::KINGMOVE[index],
            piece::Kind::Advisor => bit_constant::ADVISORMOVE[index],
            piece::Kind::Bishop => bit_constant::get_bishop_move(index, self.all_pieces()),
            piece::Kind::Knight => bit_constant::get_knight_move(index, self.all_pieces()),
            piece::Kind::Rook => {
                bit_constant::get_rook_move(index, self.all_pieces(), self.rotate_all_pieces())
            }
            piece::Kind::Cannon => {
                bit_constant::get_cannon_move(index, self.all_pieces(), self.rotate_all_pieces())
            }
            piece::Kind::Pawn => bit_constant::get_pawn_move(color == self.bottom_color, index),
            _ => 0,
        };

        // 去掉同色棋子
        result ^ (result & self.color_pieces(color))
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
        self.get_move_from_bitatom(self.bit_pieces[color as usize][kind as usize])
    }

    fn get_move_from_color(&self, color: piece::Color) -> bit_constant::BitAtom {
        self.get_move_from_bitatom(self.color_pieces(color))
    }

    fn is_killed(&self, color: piece::Color) -> bool {
        let other_color = piece::other_color(color);
        let king_bitatom = self.bit_pieces[color as usize][piece::Kind::King as usize];
        let otherking_bitatom = self.bit_pieces[other_color as usize][piece::Kind::King as usize];
        let king_face = || {
            let king_indexs =
                bit_constant::get_indexs_from_bitatom(king_bitatom | otherking_bitatom);
            assert_eq!(king_indexs.len(), 2);

            let top_king_index = king_indexs[0];
            let bottom_king_index = king_indexs[1];
            if !crate::is_same_col!(top_king_index, bottom_king_index) {
                return false;
            }

            let all_pieces = self.all_pieces();
            let mut index = top_king_index + COLCOUNT;
            while index < bottom_king_index {
                if all_pieces & bit_constant::MASK[index] != 0 {
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

    pub fn do_move(&mut self, from_index: usize, to_index: usize) -> Option<piece::Kind> {
        self.operate_move(from_index, to_index, false, piece::Kind::NoKind)
    }

    pub fn undo_move(
        &mut self,
        from_index: usize,
        to_index: usize,
        eat_kind: piece::Kind,
    ) -> Option<piece::Kind> {
        self.operate_move(from_index, to_index, true, eat_kind)
    }

    fn is_valid(&self, color: piece::Color, from_index: usize, to_index: usize) -> bool {
        self.get_color(from_index).unwrap() == color && {
            let to_color = self.get_color(to_index);
            to_color.is_none() || to_color.unwrap() != color
        }
    }

    fn operate_move(
        &mut self,
        from_index: usize,
        to_index: usize,
        is_undo: bool,
        mut eat_kind: piece::Kind,
    ) -> Option<piece::Kind> {
        let move_from_index = if is_undo { to_index } else { from_index };
        let from_color_i = self.get_color(move_from_index)? as usize;
        let from_kind = self.get_kind(move_from_index);
        let from_kind_i = from_kind as usize;
        let move_bitatom = bit_constant::MASK[from_index] | bit_constant::MASK[to_index];

        // 清除原位置，置位新位置
        if !is_undo {
            eat_kind = self.get_kind(to_index);
        }

        // 设置颜色种类位棋盘
        self.bit_pieces[from_color_i][from_kind_i] ^= move_bitatom;

        self.key ^= bit_constant::ZOBRISTKEY[from_color_i][from_kind_i][from_index]
            ^ bit_constant::ZOBRISTKEY[from_color_i][from_kind_i][to_index];
        self.lock ^= bit_constant::ZOBRISTLOCK[from_color_i][from_kind_i][from_index]
            ^ bit_constant::ZOBRISTLOCK[from_color_i][from_kind_i][to_index];

        if eat_kind != piece::Kind::NoKind {
            let to_color_i = if from_color_i == 0 { 1 } else { 0 };
            let eat_kind_i = eat_kind as usize;
            let eat_bitatom = bit_constant::MASK[to_index];

            // 设置颜色种类位棋盘
            self.bit_pieces[to_color_i][eat_kind_i] ^= eat_bitatom;

            self.key ^= bit_constant::ZOBRISTKEY[to_color_i][eat_kind_i][to_index];
            self.lock ^= bit_constant::ZOBRISTLOCK[to_color_i][eat_kind_i][to_index];
        }

        Some(eat_kind)
    }

    fn get_evaluation_is_killed(
        &self,
        from_to_index: (usize, usize),
        eat_kind: piece::Kind,
    ) -> evaluation::Evaluation {
        let (_, to_index) = from_to_index;
        // 如是对方将帅的位置则直接可走，不用判断是否被将军（如加以判断，则会直接走棋吃将帅）；棋子已走，取终点位置颜色
        let is_killed =
            eat_kind != piece::Kind::King && self.is_killed(self.get_color(to_index).unwrap());
        let count = if is_killed { 0 } else { 1 };
        // 扩展，增加其他功能

        evaluation::Evaluation::from(count)
    }

    // 执行某一着后回退（保持原局面不变）
    fn get_eval_by_do_move_undo(
        &mut self,
        from_to_index: (usize, usize),
        get_evaluation: GetEvaluation,
    ) -> Option<evaluation::Evaluation> {
        let (from_index, to_index) = from_to_index;
        let eat_kind = self.do_move(from_index, to_index)?;
        let evaluation = get_evaluation(self, from_to_index, eat_kind);

        self.undo_move(from_index, to_index, eat_kind);

        Some(evaluation)
    }

    fn get_aspect_index(&mut self, from_index: usize) -> evaluation::Aspect {
        let mut aspect = evaluation::Aspect::from(from_index, evaluation::ToIndex::new());
        for to_index in bit_constant::get_indexs_from_bitatom(self.get_move_from_index(from_index))
        {
            if let Some(evaluation) = self
                .get_eval_by_do_move_undo((from_index, to_index), Self::get_evaluation_is_killed)
            {
                aspect.insert(from_index, evaluation::ToIndex::from(to_index, evaluation));
            }
        }

        aspect
    }

    fn get_aspect_bitatom(&mut self, bit_atom: bit_constant::BitAtom) -> evaluation::Aspect {
        let mut aspect = evaluation::Aspect::new();
        for from_index in bit_constant::get_indexs_from_bitatom(bit_atom) {
            aspect.append(self.get_aspect_index(from_index));
        }

        aspect
    }

    // kind == piece::Kind::NoKind，取全部种类棋子
    fn get_aspect_color_kind(
        &mut self,
        color: piece::Color,
        kind: piece::Kind,
    ) -> evaluation::Aspect {
        self.get_aspect_bitatom(match kind {
            piece::Kind::NoKind => self.color_pieces(color),
            _ => self.bit_pieces[color as usize][kind as usize],
        })
    }

    fn get_key(&self, color: piece::Color) -> u64 {
        self.key ^ bit_constant::COLORZOBRISTKEY[color as usize]
    }

    fn get_lock(&self, color: piece::Color) -> u64 {
        self.lock ^ bit_constant::COLORZOBRISTLOCK[color as usize]
    }

    fn get_zorbist(&self, color: piece::Color, aspect: evaluation::Aspect) -> evaluation::Zorbist {
        evaluation::Zorbist::from(self.get_key(color), self.get_lock(color), aspect)
    }

    pub fn get_zorbist_color(&mut self, color: piece::Color) -> evaluation::Zorbist {
        let aspect = self.get_aspect_color_kind(color, piece::Kind::NoKind);
        self.get_zorbist(color, aspect)
    }

    pub fn get_zorbist_amove(&mut self, amove: &Rc<amove::Move>) -> evaluation::Zorbist {
        let (from_index, to_index) = amove.coordpair.from_to_index();
        let color = self.get_color(from_index).unwrap();
        let mut aspect = evaluation::Aspect::new();
        if let Some(evaluation) =
            self.get_eval_by_do_move_undo((from_index, to_index), Self::get_evaluation_is_killed)
        {
            aspect.insert(from_index, evaluation::ToIndex::from(to_index, evaluation));
        }

        self.get_zorbist(color, aspect)
    }

    pub fn get_key_values(&mut self, rowcols: String) -> Vec<(u64, u64, usize, usize, usize)> {
        let mut key_values = vec![];
        let mut color = piece::Color::Red;
        for coordpair in manual_move::ManualMove::get_coordpairs_from_rowcols(&rowcols).unwrap() {
            let (from_index, to_index) = coordpair.from_to_index();
            let key = self.get_key(color); // as i64;
            let lock = self.get_lock(color); // as i64;
            if self.do_move(from_index, to_index).is_some() {
                // let from_index = from_index as i32;
                // let to_index = to_index as i32;
                key_values.push((key, lock, from_index, to_index, 1));
            }

            color = piece::other_color(color);
        }

        key_values
    }

    pub fn get_id_lock_asps(&mut self, rowcols: String) -> Vec<(u64, u64, evaluation::Aspect)> {
        let mut id_lock_asps = vec![];
        // let mut zorbist_datas = vec![];
        // let mut aspect_datas = vec![];
        // let mut evaluation_datas = vec![];
        // let mut aspect_id = 0;
        let mut color = piece::Color::Red;
        for coordpair in manual_move::ManualMove::get_coordpairs_from_rowcols(&rowcols).unwrap() {
            let (from_index, to_index) = coordpair.from_to_index();
            let id = self.get_key(color);
            let lock = self.get_lock(color);
            if self.do_move(from_index, to_index).is_some() {
                // aspect_id += 1;
                let aspect = evaluation::Aspect::from(
                    from_index,
                    evaluation::ToIndex::from(to_index, evaluation::Evaluation::from(1)),
                );
                id_lock_asps.push((id, lock, aspect));
                // let from_index = from_index as i32;
                // let to_index = to_index as i32;
                // zorbist_datas.push(models::ZorbistData { id, lock });
                // aspect_datas.push(models::AspectData {
                //     id: aspect_id,
                //     from_index,
                //     zorbist_id: id,
                // });
                // evaluation_datas.push(models::EvaluationData {
                //     to_index,
                //     count: 1,
                //     aspect_id,
                // });
            }

            color = piece::other_color(color);
        }

        id_lock_asps
        // (zorbist_datas, aspect_datas, evaluation_datas)
    }

    pub fn to_string(&mut self) -> String {
        let mut result = format!("bottom_color: {:?}\nkinds_to_chs:\n", self.bottom_color);
        for index in 0..coord::SEATCOUNT {
            result.push(piece::get_ch(self.get_color(index), self.get_kind(index)));
            if (index + 1) % 9 == 0 {
                result.push('\n');
            }
        }

        result.push_str("\ncolor_kind_pieces:\n");
        for kind_pieces in self.bit_pieces {
            result.push_str(&bit_constant::get_bitatom_array_string(&kind_pieces, false));
        }

        result.push_str("\ncolor_pieces:\n");
        result.push_str(&bit_constant::get_bitatom_array_string(
            &[
                self.color_pieces(piece::Color::Red),
                self.color_pieces(piece::Color::Black),
            ],
            false,
        ));

        result.push_str("\nall_pieces:\n");
        result.push_str(&bit_constant::get_bitatom_array_string(
            &[self.all_pieces()],
            false,
        ));

        result.push_str("\nrotate_all_pieces:\n");
        result.push_str(&bit_constant::get_bitatom_array_string(
            &[self.rotate_all_pieces()],
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
                for index in bit_constant::get_indexs_from_bitatom(bit_board.color_pieces(color)) {
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
    use crate::common;

    use super::*;

    #[test]
    #[ignore = "界面显示bit_board"]
    fn test_bit_board() {
        for (fen, board_string) in common::FEN_BOARD_STRINGS {
            let mut bit_board = BitBoard::new(&board::fen_to_pieces(fen));
            let mut result = bit_board.to_string();

            assert_eq!(board_string, result);

            // // 可变借用
            fn to_aspect_string(bit_board: &mut BitBoard) -> String {
                let mut result = format!("aspect_string:\n");
                for color in [piece::Color::Red, piece::Color::Black] {
                    let zorbist = bit_board.get_zorbist_color(color);
                    result.push_str(&zorbist.to_string());
                }

                result
            }

            result.push('\n');
            result.push_str(&to_aspect_string(&mut bit_board));

            let name = fen.split_at(3).0;
            std::fs::write(format!("tests/output/bit_board_{name}.txt"), result)
                .expect("Write Err.");
            // dbg!(bit_board);
        }
    }
}
