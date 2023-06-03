#![allow(dead_code)]

use crate::piece;
// use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub struct Coord {
    pub row: usize,
    pub col: usize,
}

pub const COLORCOUNT: usize = 2;
pub const KINDCOUNT: usize = 7;

pub const ROWCOUNT: usize = 10;
pub const COLCOUNT: usize = 9;
pub const SEATCOUNT: usize = ROWCOUNT * COLCOUNT;

pub const SIDECOUNT: usize = 2;
pub const LEGSTATECOUNT: usize = 1 << 4;
pub const COLSTATECOUNT: usize = 1 << ROWCOUNT;
pub const ROWSTATECOUNT: usize = 1 << COLCOUNT;

pub type ZobristSeatArray = [[[u64; SEATCOUNT]; KINDCOUNT]; COLORCOUNT];
pub type ZobristSideArray = [u64; COLORCOUNT];

pub type BitAtom = u128;
pub type IndexArray = [usize; SEATCOUNT];
pub type CoordArray = [Coord; SEATCOUNT];
pub type SideBoardArray = [BitAtom; SIDECOUNT];
pub type SeatBoardArray = [BitAtom; SEATCOUNT];
pub type LegStateSeatBoardArray = [[BitAtom; SEATCOUNT]; LEGSTATECOUNT];
pub type RowStateSeatBoardArray = [[BitAtom; ROWSTATECOUNT]; COLCOUNT];
pub type ColStateSeatBoardArray = [[BitAtom; COLSTATECOUNT]; ROWCOUNT];
pub type SideSeatBoardArray = [[BitAtom; SEATCOUNT]; SIDECOUNT];

// zobrist
pub const ZOBRISTKEY: ZobristSeatArray = create_zobrist_seat_array(100);
pub const ZOBRISTLOCK: ZobristSeatArray = create_zobrist_seat_array(200);
pub const COLORZOBRISTKEY: ZobristSideArray = create_zobrist_array(300);
pub const COLORZOBRISTLOCK: ZobristSideArray = create_zobrist_array(400);
// 碰撞测试
pub const COLLIDEZOBRISTKEY: ZobristSideArray = create_zobrist_array(500);

// mask
pub const MASK: SeatBoardArray = create_mask(false);
pub const ROTATEMASK: SeatBoardArray = create_mask(true);

// put 根据所处的位置选取可放置的位置[is_bottom:0-1]
pub const KINGPUT: SideBoardArray = create_kingput();
pub const ADVISORPUT: SideBoardArray = create_advisorput();
pub const BISHOPPUT: SideBoardArray = create_bishopput();
pub const KNIGHTROOKCANNONPUT: BitAtom = 0x3f_fff_fff_fff_fff_fff_fff_fffu128;
pub const PAWNPUT: SideBoardArray = create_pawnput();

// move 帅仕根据所处的位置选取可移动位棋盘[index:0-89]
pub const KINGMOVE: SeatBoardArray = create_kingmove();
pub const ADVISORMOVE: SeatBoardArray = create_advisormove();

// 马相根据憋马腿或田心组成的四个位置状态选取可移动位棋盘[state:0-0XF][index:0-89]
pub const BISHOPMOVE: LegStateSeatBoardArray = create_bishopmove();
pub const KNIGHTMOVE: LegStateSeatBoardArray = create_knightmove();

// 车炮根据每行和每列的位置状态选取可移动位棋盘[state:0-0x1FF,0X3FF][index:0-89]
pub const ROOKROWMOVE: RowStateSeatBoardArray = create_rookcannon_row_move(false);
pub const ROOKCOLMOVE: ColStateSeatBoardArray = create_rookcannon_col_move(false);
pub const CANNONROWMOVE: RowStateSeatBoardArray = create_rookcannon_row_move(true);
pub const CANNONCOLMOVE: ColStateSeatBoardArray = create_rookcannon_col_move(true);

// 兵根据本方处于上或下的二个位置状态选取可移动位棋盘[is_bottom:0-1][index:0-89]
pub const PAWNMOVE: SideSeatBoardArray = create_pawnmove();

#[macro_export]
macro_rules! to_rowcol {
    ($index:expr) => {
        ($index / COLCOUNT, $index % COLCOUNT)
    };
}

#[macro_export]
macro_rules! to_index {
    ($row:expr, $col:expr) => {
        $row * COLCOUNT + $col
    };
}

#[macro_export]
macro_rules! is_same_col {
    ($the_index:expr,  $other_index:expr) => {
        $the_index % COLCOUNT == $other_index % COLCOUNT
    };
}

// #[macro_export]
// macro_rules! mask {
//     ($index:expr) => {
//         1u128 << $index
//     };
// }

// #[macro_export]
// macro_rules! rotate_mask {
//     ($index:expr) => {
//         1u128 << (($index % COLCOUNT) * ROWCOUNT + ($index / COLCOUNT))
//     };
// }

pub const fn get_index_array(mut bit_atom: BitAtom) -> (IndexArray, usize) {
    let mut index_array: IndexArray = [0; SEATCOUNT];
    let mut count: usize = 0;
    while bit_atom != 0 {
        let index = bit_atom.trailing_zeros() as usize;
        index_array[count] = index;

        bit_atom ^= MASK[index];
        count += 1;
    }

    (index_array, count)
}

pub fn get_index_vec(bit_atom: BitAtom) -> Vec<usize> {
    let mut indexs: Vec<usize> = Vec::new();
    let (index_array, count) = get_index_array(bit_atom);
    for idx in 0..count {
        indexs.push(index_array[idx]);
    }

    indexs
}

pub fn get_kind_put_indexs(kind: piece::Kind, is_bottom: bool) -> Vec<usize> {
    let side = if is_bottom { 1 } else { 0 };
    match kind {
        piece::Kind::King => get_index_vec(KINGPUT[side]),
        piece::Kind::Advisor => get_index_vec(ADVISORPUT[side]),
        piece::Kind::Bishop => get_index_vec(BISHOPPUT[side]),
        piece::Kind::Pawn => get_index_vec(PAWNPUT[side]),
        _ => (0..SEATCOUNT).collect(),
    }
}

const fn xorshift64(prev_value: u64) -> u64 {
    let mut next = prev_value;
    next ^= next << 13;
    next ^= next >> 7;
    next ^= next << 17;

    next
}

const fn create_zobrist_array(seed: u64) -> [u64; COLORCOUNT] {
    let mut prev_value = xorshift64(seed);
    let mut zobrist_array = [0; COLORCOUNT];
    let mut index = 0;
    while index < COLORCOUNT {
        zobrist_array[index] = prev_value;
        prev_value = xorshift64(prev_value);

        index += 1;
    }

    zobrist_array
}

const fn create_zobrist_seat_array(seed: u64) -> ZobristSeatArray {
    let mut prev_value = xorshift64(seed);
    let mut zobrist: ZobristSeatArray = [[[0; SEATCOUNT]; KINDCOUNT]; COLORCOUNT];
    let mut color = 0;
    while color < COLORCOUNT {
        let mut color_zobrist = [[0; SEATCOUNT]; KINDCOUNT];
        let mut kind = 0;
        while kind < KINDCOUNT {
            let mut kind_zobrist = [0; SEATCOUNT];
            let mut index = 0;
            while index < SEATCOUNT {
                kind_zobrist[index] = prev_value;
                prev_value = xorshift64(prev_value);

                index += 1;
            }

            color_zobrist[kind] = kind_zobrist;
            kind += 1;
        }

        zobrist[color] = color_zobrist;
        color += 1;
    }

    zobrist
}

const fn create_mask(is_rotate: bool) -> SeatBoardArray {
    let mut array: SeatBoardArray = [0; SEATCOUNT];
    let mut index = 0;
    while index < array.len() {
        let offset = if is_rotate {
            let (row, col) = to_rowcol!(index);
            col * ROWCOUNT + row
        } else {
            index
        };
        array[index] = 1 << offset;

        index += 1;
    }

    array
}

const fn create_kingput() -> SideBoardArray {
    let mut array: SideBoardArray = [0; SIDECOUNT];
    let mut side = 0;
    while side < array.len() {
        let mut index = 0;
        while index < SEATCOUNT {
            let (row, col) = to_rowcol!(index);
            let is_bottom = index >= SEATCOUNT / 2;
            let side = if is_bottom { 1 } else { 0 };
            if (row < 3 || row > 6) && (col > 2 && col < 6) {
                array[side] |= MASK[index];
            }

            index += 1;
        }

        side += 1;
    }

    array
}

const fn create_advisorput() -> SideBoardArray {
    let mut array: SideBoardArray = [0; SIDECOUNT];
    let mut side = 0;
    while side < array.len() {
        let mut index = 0;
        while index < SEATCOUNT {
            let (row, col) = to_rowcol!(index);
            let is_bottom = index >= SEATCOUNT / 2;
            let side = if is_bottom { 1 } else { 0 };
            if ((row == 0 || row == 2 || row == 7 || row == 9) && (col == 3 || col == 5))
                || ((row == 1 || row == 8) && col == 4)
            {
                array[side] |= MASK[index];
            }

            index += 1;
        }

        side += 1;
    }

    array
}

const fn create_bishopput() -> SideBoardArray {
    let mut array: SideBoardArray = [0; SIDECOUNT];
    let mut side = 0;
    while side < array.len() {
        let mut index = 0;
        while index < SEATCOUNT {
            let (row, col) = to_rowcol!(index);
            let is_bottom = index >= SEATCOUNT / 2;
            let side = if is_bottom { 1 } else { 0 };
            if ((row == 0 || row == 4 || row == 5 || row == 9) && (col == 2 || col == 6))
                || ((row == 2 || row == 7) && (col == 0 || col == 4 || col == 8))
            {
                array[side] |= MASK[index];
            }

            index += 1;
        }

        side += 1;
    }

    array
}

const fn create_pawnput() -> SideBoardArray {
    let mut array: SideBoardArray = [0; SIDECOUNT];
    let mut side = 0;
    while side < array.len() {
        let mut index = 0;
        while index < SEATCOUNT {
            let (row, col) = to_rowcol!(index);
            let mut side = 0;
            while side < SIDECOUNT {
                if (side == 1
                    && (row < 5
                        || ((row == 5 || row == 6)
                            && (col == 0 || col == 2 || col == 4 || col == 6 || col == 8))))
                    || (side == 0
                        && (row > 4
                            || ((row == 3 || row == 4)
                                && (col == 0 || col == 2 || col == 4 || col == 6 || col == 8))))
                {
                    array[side] |= MASK[index];
                }

                side += 1;
            }

            index += 1;
        }

        side += 1;
    }

    array
}

const fn create_kingmove() -> SeatBoardArray {
    let mut array: SeatBoardArray = [0; SEATCOUNT];
    let (index_array, count) = get_index_array(KINGPUT[0] | KINGPUT[1]);
    let mut valid_index: usize = 0;
    while valid_index < count {
        let index = index_array[valid_index];
        let (row, col) = to_rowcol!(index);
        array[index] = if col > 3 { MASK[index - 1] } else { 0 }
            | if col < 5 { MASK[index + 1] } else { 0 }
            | if row == 1 || row == 2 || row == 8 || row == 9 {
                MASK[index - COLCOUNT]
            } else {
                0
            }
            | if row == 0 || row == 1 || row == 7 || row == 8 {
                MASK[index + COLCOUNT]
            } else {
                0
            };

        valid_index += 1;
    }

    array
}

const fn create_advisormove() -> SeatBoardArray {
    let mut array: SeatBoardArray = [0; SEATCOUNT];
    let (index_array, count) = get_index_array(ADVISORPUT[0] | ADVISORPUT[1]);
    let mut valid_index: usize = 0;
    while valid_index < count {
        let index = index_array[valid_index];
        let (row, col) = to_rowcol!(index);
        array[index] = if col == 4 {
            MASK[index - COLCOUNT - 1]
                | MASK[index - COLCOUNT + 1]
                | MASK[index + COLCOUNT - 1]
                | MASK[index + COLCOUNT + 1]
        } else {
            MASK[if row < 3 { 13 } else { 76 }]
        };

        valid_index += 1;
    }

    array
}

const fn create_bishopmove() -> LegStateSeatBoardArray {
    let mut array: LegStateSeatBoardArray = [[0; SEATCOUNT]; LEGSTATECOUNT];
    let mut state = 0;
    while state < LEGSTATECOUNT {
        let mut all_move = [0u128; SEATCOUNT];
        let (index_array, count) = get_index_array(BISHOPPUT[0] | BISHOPPUT[1]);
        let mut valid_index: usize = 0;
        while valid_index < count {
            let index = index_array[valid_index];
            let (row, col) = to_rowcol!(index);

            let real_state = state
                | if row == 0 || row == 5 {
                    0b1100
                } else if row == 4 || row == ROWCOUNT - 1 {
                    0b0011
                } else {
                    0
                }
                | if col == 0 {
                    0b1010
                } else if col == COLCOUNT - 1 {
                    0b0101
                } else {
                    0
                };

            all_move[index] = if 0 == (real_state & 0b1000) {
                MASK[index - 2 * COLCOUNT - 2]
            } else {
                0
            } | if 0 == (real_state & 0b0100) {
                MASK[index - 2 * COLCOUNT + 2]
            } else {
                0
            } | if 0 == (real_state & 0b0010) {
                MASK[index + 2 * COLCOUNT - 2]
            } else {
                0
            } | if 0 == (real_state & 0b0001) {
                MASK[index + 2 * COLCOUNT + 2]
            } else {
                0
            };

            valid_index += 1;
        }

        array[state] = all_move;
        state += 1;
    }

    array
}

const fn create_knightmove() -> LegStateSeatBoardArray {
    let mut array: LegStateSeatBoardArray = [[0; SEATCOUNT]; LEGSTATECOUNT];
    let mut state = 0;
    while state < LEGSTATECOUNT {
        let mut all_move = [0u128; SEATCOUNT];
        let mut index = 0;
        while index < SEATCOUNT {
            let (row, col) = to_rowcol!(index);
            let real_state = state
                | if row == 0 {
                    0b1000
                } else if row == ROWCOUNT - 1 {
                    0b0001
                } else {
                    0
                }
                | if col == 0 {
                    0b0100
                } else if col == COLCOUNT - 1 {
                    0b0010
                } else {
                    0
                };

            all_move[index] = if 0 == (real_state & 0b1000) && row > 1 {
                (if col > 0 {
                    MASK[index - 2 * COLCOUNT - 1]
                } else {
                    0
                }) | if col < COLCOUNT - 1 {
                    MASK[index - 2 * COLCOUNT + 1]
                } else {
                    0
                }
            } else {
                0
            } | if 0 == (real_state & 0b0100) && col > 1 {
                (if row > 0 {
                    MASK[index - COLCOUNT - 2]
                } else {
                    0
                }) | if row < ROWCOUNT - 1 {
                    MASK[index + COLCOUNT - 2]
                } else {
                    0
                }
            } else {
                0
            } | if 0 == (real_state & 0b0010) && col < COLCOUNT - 2 {
                (if row > 0 {
                    MASK[index - COLCOUNT + 2]
                } else {
                    0
                }) | if row < ROWCOUNT - 1 {
                    MASK[index + COLCOUNT + 2]
                } else {
                    0
                }
            } else {
                0
            } | if 0 == (real_state & 0b0001) && row < ROWCOUNT - 2 {
                (if col > 0 {
                    MASK[index + 2 * COLCOUNT - 1]
                } else {
                    0
                }) | if col < COLCOUNT - 1 {
                    MASK[index + 2 * COLCOUNT + 1]
                } else {
                    0
                }
            } else {
                0
            };

            index += 1;
        }

        array[state] = all_move;
        state += 1;
    }

    array
}

//
const fn get_match_value(state: usize, row_col: usize, is_cannon: bool, is_col: bool) -> BitAtom {
    let mut match_value = 0;
    let mut direction = -1;
    while direction < 2 {
        let end_index: i32 = if direction == 1 {
            (if is_col { ROWCOUNT } else { COLCOUNT }) as i32 - 1
        } else {
            0
        }; // 每行列数或每列行数

        let mut skip = false; // 炮是否已跳
        let mut idx = direction * (row_col as i32 + direction);
        while idx <= end_index {
            let index = direction * idx;
            let has_piece = (state & 1 << index) != 0;
            if is_cannon {
                if !skip {
                    if has_piece {
                        skip = true;
                    } else {
                        match_value |= 1 << index;
                    }
                } else if has_piece {
                    match_value |= 1 << index;
                    break;
                }
            } else {
                match_value |= 1 << index;
                if has_piece {
                    break;
                }
            }

            idx += 1;
        }

        direction += 2;
    }

    match_value
}

const fn create_rookcannon_row_move(is_cannon: bool) -> RowStateSeatBoardArray {
    let mut array: RowStateSeatBoardArray = [[0; ROWSTATECOUNT]; COLCOUNT];
    let mut col = 0;
    while col < COLCOUNT {
        let mut state_move = [0u128; ROWSTATECOUNT];
        let mut state = 0;
        while state < ROWSTATECOUNT {
            // 本状态当前行或列位置有棋子
            if 0 != (state & 1 << col) {
                state_move[state] = get_match_value(state, col, is_cannon, false);
            }

            state += 1;
        }

        array[col] = state_move;
        col += 1;
    }

    array
}

const fn create_rookcannon_col_move(is_cannon: bool) -> ColStateSeatBoardArray {
    let mut array: ColStateSeatBoardArray = [[0; COLSTATECOUNT]; ROWCOUNT];
    let mut row = 0;
    while row < ROWCOUNT {
        let mut state_move = [0u128; COLSTATECOUNT];
        let mut state = 0;
        while state < COLSTATECOUNT {
            // 本状态当前行或列位置有棋子
            if 0 != (state & 1 << row) {
                let match_value = get_match_value(state, row, is_cannon, true);
                let mut col_match_value = 0u128;
                let (index_array, count) = get_index_array(match_value);
                let mut valid_index: usize = 0;
                while valid_index < count {
                    // 每行的首列置位
                    col_match_value |= MASK[index_array[valid_index] * COLCOUNT];

                    valid_index += 1;
                }

                state_move[state] = col_match_value;
            }

            state += 1;
        }

        array[row] = state_move;
        row += 1;
    }

    array
}

const fn create_pawnmove() -> SideSeatBoardArray {
    let mut array: SideSeatBoardArray = [[0; SEATCOUNT]; SIDECOUNT];
    let mut side = 0;
    while side < SIDECOUNT {
        let mut all_move = [0u128; SEATCOUNT];
        let (index_array, count) = get_index_array(PAWNPUT[side]);
        let mut valid_index: usize = 0;
        while valid_index < count {
            let index = index_array[valid_index];
            let (row, col) = to_rowcol!(index);
            all_move[index] = if (side == 0 && row > 4) || (side == 1 && row < 5) {
                (if col != 0 { MASK[index - 1] } else { 0 })
                    | if col != (COLCOUNT - 1) {
                        MASK[index + 1]
                    } else {
                        0
                    }
            } else {
                0
            } | if side == 0 && row != ROWCOUNT - 1 {
                MASK[index + COLCOUNT]
            } else if side == 1 && row != 0 {
                MASK[index - COLCOUNT]
            } else {
                0
            };

            valid_index += 1;
        }

        array[side] = all_move;
        side += 1;
    }

    array
}

pub fn get_bishop_move(index: usize, all_pieces: BitAtom) -> BitAtom {
    let (row, col) = to_rowcol!(index);
    let is_top = row == 0 || row == 5;
    let is_bottom = row == 4 || row == ROWCOUNT - 1;
    let is_left = col == 0;
    let is_right = col == COLCOUNT - 1;
    let state = (if is_top || is_left || (all_pieces & MASK[index - COLCOUNT - 1]) != 0 {
        0b1000
    } else {
        0
    }) | (if is_top || is_right || (all_pieces & MASK[index - COLCOUNT + 1]) != 0 {
        0b0100
    } else {
        0
    }) | (if is_bottom || is_left || (all_pieces & MASK[index + COLCOUNT - 1]) != 0 {
        0b0010
    } else {
        0
    }) | (if is_bottom || is_right || (all_pieces & MASK[index + COLCOUNT + 1]) != 0 {
        0b0001
    } else {
        0
    });

    return BISHOPMOVE[state][index];
}

pub fn get_knight_move(index: usize, all_pieces: BitAtom) -> BitAtom {
    let (row, col) = to_rowcol!(index);
    let state = (if row == 0 || (all_pieces & MASK[index - COLCOUNT]) != 0 {
        0b1000
    } else {
        0
    }) | (if col == 0 || (all_pieces & MASK[index - 1]) != 0 {
        0b0100
    } else {
        0
    }) | (if col == COLCOUNT - 1 || (all_pieces & MASK[index + 1]) != 0 {
        0b0010
    } else {
        0
    }) | (if row == ROWCOUNT - 1 || (all_pieces & MASK[index + COLCOUNT]) != 0 {
        0b0001
    } else {
        0
    });

    return KNIGHTMOVE[state][index];
}

pub fn get_rook_move(index: usize, all_pieces: BitAtom, rotate_all_pieces: BitAtom) -> BitAtom {
    let (row, col) = to_rowcol!(index);
    let row_offset = row * COLCOUNT;

    return (ROOKROWMOVE[col][((all_pieces >> row_offset) & 0x1FF) as usize] << row_offset)
        // 每行首列置位全体移动数列
        | (ROOKCOLMOVE[row][((rotate_all_pieces >> col * ROWCOUNT) & 0x3FF) as usize] << col);
}

pub fn get_cannon_move(index: usize, all_pieces: BitAtom, rotate_all_pieces: BitAtom) -> BitAtom {
    let (row, col) = to_rowcol!(index);
    let row_offset = row * COLCOUNT;

    return (CANNONROWMOVE[col][((all_pieces >> row_offset) & 0x1FF) as usize] << row_offset)
        // 每行首列置位全体移动数列
        | (CANNONCOLMOVE[row][((rotate_all_pieces >> col * ROWCOUNT) & 0x3FF) as usize] << col);
}

pub fn get_pawn_move(is_bottom: bool, index: usize) -> BitAtom {
    PAWNMOVE[if is_bottom { 1 } else { 0 }][index]
}

pub fn get_bitatom_array_string(boards: &[BitAtom], is_rotate: bool) -> String {
    let row_count = if is_rotate { COLCOUNT } else { ROWCOUNT };
    let col_count = if is_rotate { ROWCOUNT } else { COLCOUNT };
    let mode_value = if is_rotate { 0x3FFu128 } else { 0x1FFu128 };
    let get_board_string = |board| {
        let get_rowcol_string = |board| {
            let mut result = String::new();
            for col in 0..col_count {
                result.push(if (board & (1 << col)) == 0 { '-' } else { '1' });
            }

            result + " "
        };

        let mut result: Vec<String> = Vec::new();
        for row in 0..row_count {
            let offset = row * col_count;
            result.push(get_rowcol_string((board >> offset) & mode_value));
        }

        result
    };

    // 设置每行列数,标题行
    let length = boards.len();
    let col_per_row = length.min(col_count);
    let title = if is_rotate {
        "ABCDEFGHIJ "
    } else {
        "ABCDEFGHI "
    };
    let title_line = "   ".to_owned() + &title.repeat(col_per_row) + "\n";

    let mut result = String::new();
    let mut non_zero_count = 0;
    let mut index = 0;
    while index < length {
        let mut result_group: Vec<Vec<String>> = Vec::new();
        let mut col = 0;
        while col < col_count && index + col < length {
            let board = boards[index + col];
            if board != 0 {
                non_zero_count += 1;
            }
            result_group.push(get_board_string(board));
            col += 1;
        }

        let mut row_result = title_line.clone();
        for row in 0..result_group[0].len() {
            let row_str = format!("{row}: ");
            row_result += &row_str;
            let mut col = 0;
            while col < result_group.len() && index + col < length {
                row_result += &result_group[col][row];
                col += 1;
            }

            row_result += "\n";
        }

        result += &row_result;

        index += col_count;
    }

    let length_str = format!("length: {length} \tnon_zero: {non_zero_count}\n");
    result + &length_str
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_board_array_string(name: &str, boards: &[BitAtom], is_rotate: bool) {
        let lenght = boards.len();
        let result = format!("{name}: {lenght}\n") + &get_bitatom_array_string(boards, is_rotate);
        std::fs::write(format!("tests/constant/{name}.txt"), result).expect("Write Err.");
    }

    #[test]
    #[ignore = "此测试将全部常量输出至文本文件，以备核查。"]
    fn test_constant() {
        // zobrist
        let mut result = format!("zorbist_array:\n");
        result.push_str(&format!("COLORZOBRISTKEY: {COLORZOBRISTKEY:016x?}\n"));
        result.push_str(&format!("COLORZOBRISTLOCK: {COLORZOBRISTLOCK:016x?}\n"));
        result.push_str(&format!("COLLIDEZOBRISTKEY: {COLLIDEZOBRISTKEY:016x?}\n"));
        result.push('\n');
        result.push_str(&format!("ZOBRISTKEY: {ZOBRISTKEY:016x?}\n"));
        result.push('\n');
        result.push_str(&format!("ZOBRISTLOCK: {ZOBRISTLOCK:016x?}\n"));
        result.push('\n');
        std::fs::write("tests/constant/zorbist.txt", result).expect("Write Err.");

        // mask
        write_board_array_string("MASK", &MASK, false);
        write_board_array_string("ROTATEMASK", &ROTATEMASK, true);

        // put
        write_board_array_string("KINGPUT", &KINGPUT, false);
        write_board_array_string("ADVISORPUT", &ADVISORPUT, false);
        write_board_array_string("BISHOPPUT", &BISHOPPUT, false);
        let boards: Vec<BitAtom> = vec![KNIGHTROOKCANNONPUT];
        write_board_array_string("KNIGHTROOKCANNONPUT", &boards, false);
        write_board_array_string("PAWNPUT", &PAWNPUT, false);

        // move
        write_board_array_string("KINGMOVE", &KINGMOVE, false);
        write_board_array_string("ADVISORMOVE", &ADVISORMOVE, false);
        for index in 0..LEGSTATECOUNT {
            let name = format!("BISHOPMOVE[{index}]");
            write_board_array_string(&name, &BISHOPMOVE[index], false);

            let name = format!("KNIGHTMOVE[{index}]");
            write_board_array_string(&name, &KNIGHTMOVE[index], false);
        }
        for index in 0..COLCOUNT {
            let name = format!("ROOKROWMOVE[{index}]");
            write_board_array_string(&name, &ROOKROWMOVE[index], false);

            let name = format!("CANNONROWMOVE[{index}]");
            write_board_array_string(&name, &CANNONROWMOVE[index], false);
        }
        for index in 0..ROWCOUNT {
            let name = format!("ROOKCOLMOVE[{index}]");
            write_board_array_string(&name, &ROOKCOLMOVE[index], false);

            let name = format!("CANNONCOLMOVE[{index}]");
            write_board_array_string(&name, &CANNONCOLMOVE[index], false);
        }
        for index in 0..SIDECOUNT {
            let name = format!("PAWNMOVE[{index}]");
            write_board_array_string(&name, &PAWNMOVE[index], false);
        }

        // let (row, col) = to_rowcol!(89);
        // print!("to_rowcol: ({row},{col})");
    }
}
