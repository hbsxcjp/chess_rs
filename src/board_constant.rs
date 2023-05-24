#![allow(dead_code)]

use rand::Rng;

#[derive(Clone, Copy)]
pub struct Coord {
    row: usize,
    col: usize,
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

pub type ZobristArray = [[[u64; SEATCOUNT]; KINDCOUNT]; COLORCOUNT];

pub type BitBoard = u128;
pub type IndexArray = [usize; SEATCOUNT];
pub type CoordArray = [Coord; SEATCOUNT];
pub type SideBoardArray = [BitBoard; SIDECOUNT];
pub type SeatBoardArray = [BitBoard; SEATCOUNT];
pub type LegStateSeatBoardArray = [[BitBoard; SEATCOUNT]; LEGSTATECOUNT];
pub type RowColStateBoardArray = [[BitBoard; COLSTATECOUNT]; ROWCOUNT];
pub type ColRowStateBoardArray = [[BitBoard; ROWSTATECOUNT]; COLCOUNT];
pub type SideSeatBoardArray = [[BitBoard; SEATCOUNT]; SIDECOUNT];

// zobrist
// static ZOBRISTKEY: ZobristArray = [[[0; SEATCOUNT]; KINDCOUNT]; COLORCOUNT];
// static ZOBRISTLOCK: ZobristArray = [[[0; SEATCOUNT]; KINDCOUNT]; COLORCOUNT];
// static COLORZOBRISTKEY: &[u64] = &ZOBRISTKEY[0][0][0..2];
// static COLORZOBRISTLOCK: &[u64] = &ZOBRISTLOCK[0][0][0..2];
// static COLLIDEZOBRISTKEY: &[u64] = &ZOBRISTKEY[1][0][0..3];

// mask
pub const COORDS: CoordArray = create_coords();
pub const MASK: SeatBoardArray = create_mask();
pub const ROTATEMASK: SeatBoardArray = create_rotatemask();

// put 根据所处的位置选取可放置的位置[is_bottom:0-1]
pub const KINGPUT: SideBoardArray = create_kingput();
pub const ADVISORPUT: SideBoardArray = create_advisorput();
pub const BISHOPPUT: SideBoardArray = create_bishopput();
pub const KNIGHTROOKCANNONPUT: BitBoard = 0x3f_fff_fff_fff_fff_fff_fff_fffu128;
pub const PAWNPUT: SideBoardArray = create_pawnput();

// move 帅仕根据所处的位置选取可移动位棋盘[index:0-89]
pub const KINGMOVE: SeatBoardArray = create_kingmove();
pub const ADVISORMOVE: SeatBoardArray = create_advisormove();

// 马相根据憋马腿或田心组成的四个位置状态选取可移动位棋盘[state:0-0XF][index:0-89]
pub const BISHOPMOVE: LegStateSeatBoardArray = create_bishopmove();
pub const KNIGHTMOVE: LegStateSeatBoardArray = create_knightmove();

// 车炮根据每行和每列的位置状态选取可移动位棋盘[state:0-0x1FF,0X3FF][index:0-89]
pub const ROOKROWMOVE: ColRowStateBoardArray = create_rookcannon_row_move(false);
pub const ROOKCOLMOVE: RowColStateBoardArray = create_rookcannon_col_move(false);
pub const CANNONROWMOVE: ColRowStateBoardArray = create_rookcannon_row_move(true);
pub const CANNONCOLMOVE: RowColStateBoardArray = create_rookcannon_col_move(true);

// 兵根据本方处于上或下的二个位置状态选取可移动位棋盘[is_bottom:0-1][index:0-89]
pub const PAWNMOVE: SideSeatBoardArray = create_pawnmove();

pub fn create_zobrist() -> ZobristArray {
    let mut zobrist: ZobristArray = [[[0; SEATCOUNT]; KINDCOUNT]; COLORCOUNT];
    for color in 0..COLORCOUNT {
        let mut color_zobrist = [[0; SEATCOUNT]; KINDCOUNT];
        for kind in 0..KINDCOUNT {
            let mut kind_zobrist = [0; SEATCOUNT];
            for index in 0..SEATCOUNT {
                kind_zobrist[index] = rand::thread_rng().gen_range(u64::MIN..=u64::MAX);
            }

            color_zobrist[kind] = kind_zobrist;
        }

        zobrist[color] = color_zobrist;
    }

    return zobrist;
}

const fn create_coords() -> CoordArray {
    let mut coords: CoordArray = [Coord { row: 0, col: 0 }; SEATCOUNT];

    let mut index = 0;
    while index < coords.len() {
        coords[index] = Coord {
            row: index / COLCOUNT,
            col: index % COLCOUNT,
        };
        index += 1;
    }

    coords
}

const fn create_mask() -> SeatBoardArray {
    let mut array: SeatBoardArray = [0; SEATCOUNT];
    let mut index = 0;
    while index < array.len() {
        array[index] = 1 << index;
        index += 1;
    }

    array
}

const fn create_rotatemask() -> SeatBoardArray {
    let mut array: SeatBoardArray = [0; SEATCOUNT];
    let mut index = 0;
    while index < array.len() {
        let coord = COORDS[index];
        array[index] = 1 << (coord.col * ROWCOUNT + coord.row);
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
            let coord = COORDS[index];
            let row = coord.row;
            let col = coord.col;
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
            let coord = COORDS[index];
            let row = coord.row;
            let col = coord.col;
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
            let coord = COORDS[index];
            let row = coord.row;
            let col = coord.col;
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
            let coord = COORDS[index];
            let row = coord.row;
            let col = coord.col;
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

const fn get_one_index_array(mut board: BitBoard) -> (IndexArray, usize) {
    let mut index_array: IndexArray = [0; SEATCOUNT];
    let mut count: usize = 0;
    while board != 0 {
        let index = board.trailing_zeros() as usize;
        index_array[count] = index;

        board ^= MASK[index];
        count += 1;
    }

    (index_array, count)
}

const fn create_kingmove() -> SeatBoardArray {
    let mut array: SeatBoardArray = [0; SEATCOUNT];
    let (index_array, count) = get_one_index_array(KINGPUT[0] | KINGPUT[1]);
    let mut valid_index: usize = 0;
    while valid_index < count {
        let index = index_array[valid_index];
        let coord = COORDS[index];
        let row = coord.row;
        let col = coord.col;

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
    let (index_array, count) = get_one_index_array(ADVISORPUT[0] | ADVISORPUT[1]);
    let mut valid_index: usize = 0;
    while valid_index < count {
        let index = index_array[valid_index];
        let coord = COORDS[index];
        let row = coord.row;
        let col = coord.col;

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
        let (index_array, count) = get_one_index_array(BISHOPPUT[0] | BISHOPPUT[1]);
        let mut valid_index: usize = 0;
        while valid_index < count {
            let index = index_array[valid_index];
            let coord = COORDS[index];
            let row = coord.row;
            let col = coord.col;

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
        while index < COORDS.len() {
            let coord = COORDS[index];
            let row = coord.row;
            let col = coord.col;

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
const fn get_match_value(state: usize, index: usize, is_cannon: bool, is_col: bool) -> BitBoard {
    let mut match_value = 0;
    let mut is_high = 0;
    while is_high < 2 {
        let direction: i32 = if is_high == 1 { 1 } else { -1 };
        let end_index: i32 = if is_high == 1 {
            (if is_col { ROWCOUNT } else { COLCOUNT }) as i32 - 1
        } else {
            0
        }; // 每行列数或每列行数

        let mut skip = false; // 炮是否已跳
        let mut idx = direction * (index as i32 + direction);
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
                if has_piece
                // 遇到棋子
                {
                    break;
                }
            }

            idx += 1;
        }

        is_high += 1;
    }

    return match_value;
}

const fn create_rookcannon_col_move(is_cannon: bool) -> RowColStateBoardArray {
    let mut array: RowColStateBoardArray = [[0; COLSTATECOUNT]; ROWCOUNT];
    let mut index = 0;
    while index < ROWCOUNT {
        let mut state_move = [0u128; COLSTATECOUNT];
        let mut state = 0;
        while state < COLSTATECOUNT {
            // 本状态当前行或列位置有棋子
            if 0 != (state & 1 << index) {
                let match_value = get_match_value(state, index, is_cannon, true);
                let mut col_match_value = 0u128;
                let mut row = 0;
                while row < ROWCOUNT {
                    if 0 != (match_value & 1 << row) {
                        // 每行的首列置位
                        col_match_value |= MASK[row * COLCOUNT];
                    }

                    row += 1;
                }

                state_move[state] = col_match_value;
            }

            state += 1;
        }

        array[index] = state_move;
        index += 1;
    }

    array
}

const fn create_rookcannon_row_move(is_cannon: bool) -> ColRowStateBoardArray {
    let mut array: ColRowStateBoardArray = [[0; ROWSTATECOUNT]; COLCOUNT];
    let mut index = 0;
    while index < COLCOUNT {
        let mut state_move = [0u128; ROWSTATECOUNT];
        let mut state = 0;
        while state < ROWSTATECOUNT {
            // 本状态当前行或列位置有棋子
            if 0 != (state & 1 << index) {
                state_move[state] = get_match_value(state, index, is_cannon, false);
            }

            state += 1;
        }

        array[index] = state_move;
        index += 1;
    }

    array
}

const fn create_pawnmove() -> SideSeatBoardArray {
    let mut array: SideSeatBoardArray = [[0; SEATCOUNT]; SIDECOUNT];
    let mut side = 0;
    while side < SIDECOUNT {
        let mut all_move = [0u128; SEATCOUNT];
        let (index_array, count) = get_one_index_array(PAWNPUT[side]);
        let mut valid_index: usize = 0;
        while valid_index < count {
            let index = index_array[valid_index];
            let coord = COORDS[index];
            let row = coord.row;
            let col = coord.col;

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

pub fn get_bishop_move(index: usize, all_pieces: BitBoard) -> BitBoard {
    let coord = COORDS[index];
    let row = coord.row;
    let col = coord.col;
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

pub fn get_knight_move(index: usize, all_pieces: BitBoard) -> BitBoard {
    let coord = COORDS[index];
    let row = coord.row;
    let col = coord.col;
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

pub fn get_rookcannon_move(
    is_cannon: bool,
    index: usize,
    all_pieces: BitBoard,
    rotate_pieces: BitBoard,
) -> BitBoard {
    let coord = COORDS[index];
    let row = coord.row;
    let col = coord.col;
    let row_offset = row * COLCOUNT;
    let row_move = if is_cannon {
        CANNONCOLMOVE
    } else {
        ROOKCOLMOVE
    };
    let col_move = if is_cannon {
        CANNONROWMOVE
    } else {
        ROOKROWMOVE
    };

    // 每行首列置位全体移动数列
    return (row_move[col][((all_pieces >> row_offset) & 0x1FF) as usize] << row_offset)
        | (col_move[row][((rotate_pieces >> col * ROWCOUNT) & 0x3FF) as usize] << col);
}

pub fn get_board_string(board: BitBoard) -> Vec<String> {
    fn get_rowcol_string(board: BitBoard) -> String {
        let mut result = String::new();
        for col in 0..COLCOUNT {
            result += if (board & (1 << col)) == 0 { "-" } else { "1" };
        }

        return result + " ";
    }

    let mut result: Vec<String> = vec![];
    for row in 0..ROWCOUNT {
        let offset = row * COLCOUNT;
        result.push(get_rowcol_string((board & (0x1FF << offset)) >> offset));
    }

    return result;
}

pub fn write_board_array_string(name: &str, boards: &[BitBoard]) {
    // 设置每行列数,标题行
    let length = boards.len();

    let mut title_line = String::from("   ");
    let col_per_row = if length < COLCOUNT { length } else { COLCOUNT };
    for _ in 0..col_per_row {
        title_line += "ABCDEFGHI ";
    }
    title_line += "\n";

    let mut result = format!("{name}: {length}\n");
    let mut non_zero_count = 0;
    let mut index = 0;
    while index < length {
        let mut result_group: Vec<Vec<String>> = vec![];
        let mut col = 0;
        while col < COLCOUNT && index + col < length {
            let board = boards[index + col];
            if board != 0 {
                non_zero_count += 1;
            }
            result_group.push(get_board_string(board));
            col += 1;
        }

        let mut row_result = title_line.clone();
        for row in 0..ROWCOUNT {
            let row_str = format!("{row}: ");
            row_result += &row_str;
            let mut col = 0;
            while col < COLCOUNT && index + col < length {
                row_result += &result_group[col][row];
                col += 1;
            }

            row_result += "\n";
        }

        result += &row_result;

        index += COLCOUNT;
    }

    let length_str = format!("length: {length}\nnon_zero: {non_zero_count}\n");
    result += &length_str;

    std::fs::write(format!("tests/{name}.txt"), result).expect("Write Err.");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constant() {
        // mask
        write_board_array_string("MASK", &MASK);
        write_board_array_string("ROTATEMASK", &ROTATEMASK);

        // put
        write_board_array_string("KINGPUT", &KINGPUT);
        write_board_array_string("ADVISORPUT", &ADVISORPUT);
        write_board_array_string("BISHOPPUT", &BISHOPPUT);
        let boards: Vec<BitBoard> = vec![KNIGHTROOKCANNONPUT];
        write_board_array_string("KNIGHTROOKCANNONPUT", &boards);
        write_board_array_string("PAWNPUT", &PAWNPUT);

        // move
        write_board_array_string("KINGMOVE", &KINGMOVE);
        write_board_array_string("ADVISORMOVE", &ADVISORMOVE);
        for index in 0..LEGSTATECOUNT {
            let name = format!("BISHOPMOVE[{index}]");
            write_board_array_string(&name, &BISHOPMOVE[index]);

            let name = format!("KNIGHTMOVE[{index}]");
            write_board_array_string(&name, &KNIGHTMOVE[index]);
        }
        for index in 0..COLCOUNT {
            let name = format!("ROOKROWMOVE[{index}]");
            write_board_array_string(&name, &ROOKROWMOVE[index]);

            let name = format!("CANNONROWMOVE[{index}]");
            write_board_array_string(&name, &CANNONROWMOVE[index]);
        }
        for index in 0..ROWCOUNT {
            let name = format!("ROOKCOLMOVE[{index}]");
            write_board_array_string(&name, &ROOKCOLMOVE[index]);

            let name = format!("CANNONCOLMOVE[{index}]");
            write_board_array_string(&name, &CANNONCOLMOVE[index]);
        }
        for index in 0..SIDECOUNT {
            let name = format!("PAWNMOVE[{index}]");
            write_board_array_string(&name, &PAWNMOVE[index]);
        }
    }
}
