#![allow(dead_code)]

#[derive(Clone, Copy)]
pub struct Coord {
    row: usize,
    col: usize,
}

const ROWCOUNT: usize = 10;
const COLCOUNT: usize = 9;
const SEATCOUNT: usize = ROWCOUNT * COLCOUNT;

const SIDECOUNT: usize = 2;
const LEGSTATECOUNT: usize = 1 << 4;
const COLSTATECOUNT: usize = 1 << COLCOUNT;
const ROWSTATECOUNT: usize = 1 << ROWCOUNT;

type IndexSeatArray = [usize; SEATCOUNT];

type CoordSeatArray = [Coord; SEATCOUNT];

type SideBoardArray = [u128; SIDECOUNT];

type SeatBoardArray = [u128; SEATCOUNT];

type LegStateSeatBoardArray = [[u128; SEATCOUNT]; LEGSTATECOUNT];

type ColStateSeatBoardArray = [[u128; SEATCOUNT]; COLSTATECOUNT];

type RowStateSeatBoardArray = [[u128; SEATCOUNT]; ROWSTATECOUNT];

type SideSeatBoardArray = [[u128; SEATCOUNT]; SIDECOUNT];

const COORDS: CoordSeatArray = create_coords();
const MASK: SeatBoardArray = create_mask();
const ROTATEMASK: SeatBoardArray = create_rotatemask();

// 根据所处的位置选取可放置的位置[isBottom:0-1]
const KINGPUT: SideBoardArray = create_kingput();
const ADVISORPUT: SideBoardArray = create_advisorput();
const BISHOPPUT: SideBoardArray = create_bishopput();
const KNIGHTROOKCANNONPUT: u128 = 0x3f_fff_fff_fff_fff_fff_fff_fffu128;
const PAWNPUT: SideBoardArray = create_pawnput();

// 帅仕根据所处的位置选取可移动位棋盘[index:0-89]
const KINGMOVE: SeatBoardArray = create_kingmove();
const ADVISORMOVE: SeatBoardArray = create_advisormove();

// 马相根据憋马腿或田心组成的四个位置状态选取可移动位棋盘[state:0-0XF][index:0-89]
const BISHOPMOVE: LegStateSeatBoardArray = create_bishopmove();
const KNIGHTMOVE: LegStateSeatBoardArray = create_knightmove();

// 车炮根据每行和每列的位置状态选取可移动位棋盘[state:0-0x1FF,0X3FF][index:0-89]

// private static readonly List<BigInteger[]> RookRowMove = CreateRookCannonMove(PieceKind.Rook, false);
// private static readonly List<BigInteger[]> RookColMove = CreateRookCannonMove(PieceKind.Rook, true);
// private static readonly List<BigInteger[]> CannonRowMove = CreateRookCannonMove(PieceKind.Cannon, false);
// private static readonly List<BigInteger[]> CannonColMove = CreateRookCannonMove(PieceKind.Cannon, true);

// 兵根据本方处于上或下的二个位置状态选取可移动位棋盘[isBottom:0-1][index:0-89]
const PAWNMOVE: SideSeatBoardArray = create_pawnmove();

const fn create_coords() -> CoordSeatArray {
    let mut coords: CoordSeatArray = [Coord { row: 0, col: 0 }; SEATCOUNT];

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

const fn get_one_index_array(mut board: u128) -> (IndexSeatArray, usize) {
    let mut index_array: IndexSeatArray = [0; SEATCOUNT];
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

        array[valid_index] = if col > 3 { MASK[index - 1] } else { 0 }
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

        array[valid_index] = if col == 4 {
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

            all_move[valid_index] = if 0 == (real_state & 0b1000) {
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

const fn create_pawnmove() -> SideSeatBoardArray {
    let mut array: SideSeatBoardArray = [[0; SEATCOUNT]; SIDECOUNT];
    let mut side = 0;
    while side < SIDECOUNT {
        let mut all_move = [0u128; SEATCOUNT];
        let (index_array, count) = get_one_index_array(PAWNPUT[0] | PAWNPUT[1]);
        let mut valid_index: usize = 0;
        while valid_index < count {
            let index = index_array[valid_index];
            let coord = COORDS[index];
            let row = coord.row;
            let col = coord.col;

            all_move[valid_index] = if (side == 0 && row > 4) || (side == 1 && row < 5) {
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

pub fn get_board_string(board: u128, is_rotate: bool) -> Vec<String> {
    fn get_rowcol_string(board: u128, col_num: usize) -> String {
        let mut result = String::new();
        for col in 0..col_num {
            result += if (board & (1 << col)) == 0 { "-" } else { "1" };
        }
        result += " ";

        return result;
    }

    let row_num = if is_rotate { COLCOUNT } else { ROWCOUNT };
    let col_num = if is_rotate { ROWCOUNT } else { COLCOUNT };
    let mode = if is_rotate { 0x3FF } else { 0x1FF };
    let mut result: Vec<String> = vec![];
    for row in 0..row_num {
        let offset = row * col_num;
        result.push(get_rowcol_string(
            (board & (mode << offset)) >> offset,
            col_num,
        ));
    }

    return result;
}

pub fn get_board_array_string(
    boards: &[u128],
    colnum_perrow: usize,
    show_zero: bool,
    is_rotate: bool,
) -> String {
    // 处理非零情况
    let mut nonzero_boards: Vec<u128> = vec![];
    if !show_zero {
        for index in 0..boards.len() {
            if boards[index] != 0 {
                nonzero_boards.push(boards[index]);
            }
        }
    } else {
        nonzero_boards = boards.to_vec();
    }

    // 设置每行列数,标题行
    let length = nonzero_boards.len();
    let row_num = if is_rotate { COLCOUNT } else { ROWCOUNT };
    let colnum_perrow = if length < colnum_perrow {
        length
    } else {
        colnum_perrow
    };

    let mut title_line = String::from("   ");
    for _ in 0..colnum_perrow {
        title_line += if is_rotate {
            "ABCDEFGHIJ "
        } else {
            "ABCDEFGHI "
        };
    }
    title_line += "\n";

    let mut result = String::new();
    let mut index = 0;
    while index < length {
        let mut result_perrow: Vec<Vec<String>> = vec![];
        let mut col = 0;
        while col < colnum_perrow && index + col < length {
            result_perrow.push(get_board_string(boards[index + col], is_rotate));
            col += 1;
        }

        let mut row_result = title_line.clone();
        for row in 0..row_num {
            let row_str = format!("{row}: ");
            row_result += &row_str;
            let mut col = 0;
            while col < colnum_perrow && index + col < length {
                row_result += &result_perrow[col][row];
                col += 1;
            }

            row_result += "\n";
        }

        result += &row_result;

        index += colnum_perrow;
    }
    let length_str = format!("length: {length}\n");
    result += &length_str;

    return result;
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_mask() {
        let len = MASK.len();
        let board_str = get_board_array_string(&MASK, COLCOUNT, true, false);
        let result = format!("MASK: {len}\n{board_str}\n");

        fs::write("tests/test_mask.txt", result).expect("Write Err.");
    }

    #[test]
    fn test_rotatemask() {
        let len = ROTATEMASK.len();
        let board_str = get_board_array_string(&ROTATEMASK, ROWCOUNT, true, true);
        let result = format!("ROTATEMASK: {len}\n{board_str}\n");

        fs::write("tests/test_rotatemask.txt", result).expect("Write Err.");
    }

    #[test]
    fn test_kingput() {
        let len = KINGPUT.len();
        let board_str = get_board_array_string(&KINGPUT, COLCOUNT, true, false);
        let result = format!("KINGPUT: {len}\n{board_str}\n");

        fs::write("tests/test_kingput.txt", result).expect("Write Err.");
    }

    #[test]
    fn test_advisorput() {
        let len = ADVISORPUT.len();
        let board_str = get_board_array_string(&ADVISORPUT, COLCOUNT, true, false);
        let result = format!("ADVISORPUT: {len}\n{board_str}\n");

        fs::write("tests/test_advisorput.txt", result).expect("Write Err.");
    }

    #[test]
    fn test_bishopput() {
        let len = BISHOPPUT.len();
        let board_str = get_board_array_string(&BISHOPPUT, COLCOUNT, true, false);
        let result = format!("BISHOPPUT: {len}\n{board_str}\n");

        fs::write("tests/test_bishopput.txt", result).expect("Write Err.");
    }

    #[test]
    fn test_knightrookcannonput() {
        let boards: Vec<u128> = vec![KNIGHTROOKCANNONPUT];
        let len = boards.len();
        let board_str = get_board_array_string(&boards, COLCOUNT, true, false);
        let result = format!("KNIGHTROOKCANNONPUT: {len}\n{board_str}\n");

        fs::write("tests/test_knightrookcannonput.txt", result).expect("Write Err.");
    }

    #[test]
    fn test_pawnput() {
        let len = PAWNPUT.len();
        let board_str = get_board_array_string(&PAWNPUT, COLCOUNT, true, false);
        let result = format!("PAWNPUT: {len}\n{board_str}\n");

        fs::write("tests/test_pawnput.txt", result).expect("Write Err.");
    }

    #[test]
    fn test_kingmove() {
        let len = KINGMOVE.len();
        let board_str = get_board_array_string(&KINGMOVE, COLCOUNT, false, false);
        let result = format!("KINGMOVE: {len}\n{board_str}\n");

        fs::write("tests/test_kingmove.txt", result).expect("Write Err.");
    }

    #[test]
    fn test_advisormove() {
        let len = ADVISORMOVE.len();
        let board_str = get_board_array_string(&ADVISORMOVE, 5, false, false);
        let result = format!("ADVISORMOVE: {len}\n{board_str}\n");

        fs::write("tests/test_advisormove.txt", result).expect("Write Err.");
    }
}
