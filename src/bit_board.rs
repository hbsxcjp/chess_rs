#![allow(dead_code)]
#![allow(unused_imports)]

use crate::bit_constant;
use crate::bit_constant::COLCOUNT;
use crate::board;
use crate::piece;

type GetEffect =
    fn(&BitBoard, from_index: usize, to_index: usize, eat_kind: piece::Kind) -> MoveEffect;

#[derive(Debug)]
pub struct MoveEffect {
    from_index: usize,
    to_index: usize,

    score: i32,
    frequency: i32,
}

impl MoveEffect {
    pub fn from(from_index: usize, to_index: usize, score: i32, frequency: i32) -> MoveEffect {
        MoveEffect {
            from_index,
            to_index,
            score,
            frequency,
        }
    }

    pub fn to_string(&self) -> String {
        let (frow, fcol) = crate::to_rowcol!(self.from_index);
        let (trow, tcol) = crate::to_rowcol!(self.to_index);
        let score = self.score;
        let fre = self.frequency;
        format!("[{},{}] => [{},{}] {score} {fre}\n", frow, fcol, trow, tcol)
    }
}

#[derive(Debug)]
pub struct BitBoard {
    bottom_color: piece::Color,
    colors: [piece::Color; bit_constant::SEATCOUNT],
    kinds: [piece::Kind; bit_constant::SEATCOUNT],

    // 计算中间存储数据(基本局面改动时更新)
    color_kind_pieces: [[bit_constant::BitAtom; bit_constant::KINDCOUNT]; bit_constant::COLORCOUNT],
    color_pieces: [bit_constant::BitAtom; bit_constant::COLORCOUNT],
    all_pieces: bit_constant::BitAtom,
    rotate_all_pieces: bit_constant::BitAtom,

    // 哈希局面数据
    hashkey: u64,
    // private static HistoryRecord? historyRecord;
}

impl BitBoard {
    pub fn new(pieces: &board::Pieces) -> BitBoard {
        let mut bit_board: BitBoard = BitBoard {
            bottom_color: board::get_bottom_color(&pieces),
            colors: [piece::Color::NoColor; bit_constant::SEATCOUNT],
            kinds: [piece::Kind::NoKind; bit_constant::SEATCOUNT],

            color_kind_pieces: [[0; bit_constant::KINDCOUNT]; bit_constant::COLORCOUNT],
            color_pieces: [0; bit_constant::COLORCOUNT],
            all_pieces: 0,
            rotate_all_pieces: 0,
            hashkey: 0,
        };

        let mut index = 0;
        for piece in pieces {
            match piece {
                piece::Piece::None => (),
                piece::Piece::Some(color, kind) => {
                    bit_board.colors[index] = *color;
                    bit_board.kinds[index] = *kind;

                    bit_board.color_kind_pieces[*color as usize][*kind as usize] |=
                        bit_constant::MASK[index];
                    bit_board.color_pieces[*color as usize] |= bit_constant::MASK[index];
                    bit_board.all_pieces |= bit_constant::MASK[index];
                    bit_board.rotate_all_pieces |= bit_constant::ROTATEMASK[index];
                }
            }

            index += 1;
        }

        bit_board
    }

    fn get_index_move(&self, index: usize) -> bit_constant::BitAtom {
        let color = self.colors[index];
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

    fn get_bitatom_move(&self, bit_atom: bit_constant::BitAtom) -> bit_constant::BitAtom {
        let mut result = 0;
        for index in bit_constant::get_index_vec(bit_atom) {
            result |= self.get_index_move(index);
        }

        result
    }

    fn get_color_kind_move(&self, color: piece::Color, kind: piece::Kind) -> bit_constant::BitAtom {
        self.get_bitatom_move(self.color_kind_pieces[color as usize][kind as usize])
    }

    fn get_color_move(&self, color: piece::Color) -> bit_constant::BitAtom {
        self.get_bitatom_move(self.color_pieces[color as usize])
    }

    pub fn is_killed(&self, color: piece::Color) -> bool {
        let other_color = piece::other_color(color);
        let king_bitatom = self.color_kind_pieces[color as usize][piece::Kind::King as usize];
        let otherking_bitatom =
            self.color_kind_pieces[other_color as usize][piece::Kind::King as usize];
        let king_face = || {
            let (king_index_array, count) =
                bit_constant::get_index_array(king_bitatom | otherking_bitatom);
            assert_eq!(count, 2);

            let top_king_index = king_index_array[0];
            let bottom_king_index = king_index_array[1];
            if !crate::is_same_col!(top_king_index, bottom_king_index) {
                return false;
            }

            let mut index = top_king_index + bit_constant::COLCOUNT;
            while index < bottom_king_index {
                if self.all_pieces & bit_constant::MASK[index] != 0 {
                    return false;
                }
                index += bit_constant::COLCOUNT;
            }

            true
        };

        king_face() || (self.get_color_move(other_color) & king_bitatom) != 0
    }

    pub fn is_failed(&self, color: piece::Color) -> bool {
        self.get_color_move(color) == 0
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
        let from_color = self.colors[start_index];
        let from_kind = self.kinds[start_index];
        let from_color_int = from_color as usize;
        let from_bitatrom = bit_constant::MASK[from_index];
        let to_bitatom = bit_constant::MASK[to_index];
        let move_bitatom = from_bitatrom | to_bitatom;
        if !is_back {
            eat_kind = self.kinds[to_index];
        }

        // 清除原位置，置位新位置
        self.colors[end_index] = from_color;
        self.kinds[end_index] = from_kind;
        self.colors[start_index] = piece::Color::NoColor;
        self.kinds[start_index] = piece::Kind::NoKind;

        self.color_kind_pieces[from_color_int][from_kind as usize] ^= move_bitatom;
        self.color_pieces[from_color_int] ^= move_bitatom;

        // hashkey ^= (BitConstants.ZobristKey[from_colorInt][from_kindInt][fromIndex] ^ BitConstants.ZobristKey[from_colorInt][from_kindInt][toIndex]);
        // hashLock ^= (BitConstants.ZobristLock[from_colorInt][from_kindInt][fromIndex] ^ BitConstants.ZobristLock[from_colorInt][from_kindInt][toIndex]);

        if eat_kind != piece::Kind::NoKind {
            if is_back {
                self.colors[start_index] = piece::other_color(from_color);
                self.kinds[start_index] = eat_kind;
            }
            let to_color_int = if from_color_int == 0 { 1 } else { 0 };
            self.color_kind_pieces[to_color_int][eat_kind as usize] ^= to_bitatom;
            self.color_pieces[to_color_int] ^= to_bitatom;

            // hashkey ^= BitConstants.ZobristKey[toColorInt][eatKindInt][toIndex];
            // hashLock ^= BitConstants.ZobristLock[toColorInt][eatKindInt][toIndex];

            self.all_pieces ^= from_bitatrom;
            self.rotate_all_pieces ^= bit_constant::ROTATEMASK[from_index];
        } else {
            self.all_pieces ^= move_bitatom;
            self.rotate_all_pieces ^=
                bit_constant::ROTATEMASK[from_index] | bit_constant::ROTATEMASK[to_index];
        }

        eat_kind
    }

    fn get_effect_killed(
        &self,
        from_index: usize,
        to_index: usize,
        eat_kind: piece::Kind,
    ) -> MoveEffect {
        // 如是对方将帅的位置则直接可走，不用判断是否被将军（如加以判断，则会直接走棋吃将帅）；棋子已走，取终点位置颜色
        MoveEffect::from(
            from_index,
            to_index,
            if eat_kind != piece::Kind::King && self.is_killed(self.colors[to_index]) {
                -1
            } else {
                1
            },
            0,
        )
    }

    // 执行某一着后的效果(委托函数可叠加)
    fn domove_get_effect(
        &mut self,
        from_index: usize,
        to_index: usize,
        get_effect: GetEffect,
    ) -> MoveEffect {
        let eat_kind = self.do_move(from_index, to_index, false, piece::Kind::NoKind);

        let effect = get_effect(self, from_index, to_index, eat_kind);

        self.do_move(from_index, to_index, true, eat_kind);
        effect
    }

    fn get_index_effects(&mut self, from_index: usize) -> Vec<MoveEffect> {
        let mut effects: Vec<MoveEffect> = Vec::new();
        for to_index in bit_constant::get_index_vec(self.get_index_move(from_index)) {
            effects.push(self.domove_get_effect(from_index, to_index, Self::get_effect_killed));
        }

        effects
    }

    fn get_bitatom_effects(&mut self, bit_atom: bit_constant::BitAtom) -> Vec<MoveEffect> {
        let mut effects: Vec<MoveEffect> = Vec::new();
        for from_index in bit_constant::get_index_vec(bit_atom) {
            effects.append(&mut self.get_index_effects(from_index));
        }

        effects
    }

    // kind == piece::Kind::NoKind，取全部种类棋子
    fn get_color_kind_effects(
        &mut self,
        color: piece::Color,
        kind: piece::Kind,
    ) -> Vec<MoveEffect> {
        self.get_bitatom_effects(match kind {
            piece::Kind::NoKind => self.color_pieces[color as usize],
            _ => self.color_kind_pieces[color as usize][kind as usize],
        })
    }

    fn get_color_effects(&mut self, color: piece::Color) -> Vec<MoveEffect> {
        self.get_color_kind_effects(color, piece::Kind::NoKind)
    }

    pub fn to_string(&self) -> String {
        let mut result = format!("bottom_color: {:?}\ncolor_kinds:\n", self.bottom_color);
        for index in 0..bit_constant::SEATCOUNT {
            result.push(match self.colors[index] {
                piece::Color::Red => '-',
                piece::Color::Black => '+',
                piece::Color::NoColor => '_',
            });
            result.push(piece::get_ch(&self.colors[index], &self.kinds[index]));
            result.push(' ');

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

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bit_board() {
        let fens = [
            "rnbakabnr/9/1c5c1/p1p1p1p1p/9/9/P1P1P1P1P/1C5C1/9/RNBAKABNR",
            "5a3/4ak2r/6R2/8p/9/9/9/B4N2B/4K4/3c5",
            "2b1kab2/4a4/4c4/9/9/3R5/9/1C7/4r4/2BK2B2",
            "4kab2/4a4/4b4/3N5/9/4N4/4n4/4B4/4A4/3AK1B2",
        ];

        for fen in fens {
            let mut bit_board = BitBoard::new(&board::fen_to_pieces(fen));
            let mut result = bit_board.to_string();
            result.push('\n');

            for color in [piece::Color::Red, piece::Color::Black] {
                let effects = bit_board.get_color_effects(color);
                let count = effects.len();
                for effect in effects {
                    result.push_str(&effect.to_string());
                }
                result.push_str(&format!("count: {count}\n\n"));
            }

            let name = fen.split_at(3).0;
            std::fs::write(format!("tests/{name}.txt"), result).expect("Write Err.");
            // dbg!(board);
        }

        // let (row, col) = crate::to_rowcol!(9);
        // print!("to_rowcol: ({row},{col})");
    }
}
