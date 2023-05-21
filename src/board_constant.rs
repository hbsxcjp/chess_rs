#![allow(dead_code)]

const ROWCOUNT: usize = 10;
const COLCOUNT: usize = 9;
const SEATCOUNT: usize = ROWCOUNT * COLCOUNT;

const SIDECOUNT: usize = 2;
const LEGSTATECOUNT: usize = 1 << 4;
const COLSTATECOUNT: usize = 1 << COLCOUNT;
const ROWSTATECOUNT: usize = 1 << ROWCOUNT;

#[derive(Clone, Copy)]
pub struct Coord {
    row: usize,
    col: usize,
}

type IndexSeatArray = [usize; SEATCOUNT];

type CoordSeatArray = [Coord; SEATCOUNT];

type SideBoardArray = [u128; SIDECOUNT];

type SeatBoardArray = [u128; SEATCOUNT];

type LegStateSeatBoardArray = [[u128; SEATCOUNT]; LEGSTATECOUNT];

type ColStateSeatBoardArray = [[u128; SEATCOUNT]; COLSTATECOUNT];

type RowStateSeatBoardArray = [[u128; SEATCOUNT]; ROWSTATECOUNT];

type SideSeatBoardArray = [[u128; SEATCOUNT]; SIDECOUNT];

const COORDS: CoordSeatArray = {
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
};

const MASK: SeatBoardArray = {
    let mut array: SeatBoardArray = [0; SEATCOUNT];
    let mut index = 0;
    while index < array.len() {
        array[index] = 1 << index;
        index += 1;
    }

    array
};

const ROTATEMASK: SeatBoardArray = {
    let mut array: SeatBoardArray = [0; SEATCOUNT];
    let mut index = 0;
    while index < array.len() {
        let coord = COORDS[index];
        array[index] = 1 << (coord.col * ROWCOUNT + coord.row);
        index += 1;
    }

    array
};

// 根据所处的位置选取可放置的位置[isBottom:0-1]
const KINGPUT: SideBoardArray = {
    let mut array: SideBoardArray = [0; SIDECOUNT];
    let mut index = 0;
    while index < array.len() {
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

    array
};

const ADVISORPUT: SideBoardArray = {
    let mut array: SideBoardArray = [0; SIDECOUNT];
    let mut index = 0;
    while index < array.len() {
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

    array
};

const BISHOPPUT: SideBoardArray = {
    let mut array: SideBoardArray = [0; SIDECOUNT];
    let mut index = 0;
    while index < array.len() {
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

    array
};

const KNIGHTROOKCANNONPUT: u128 = 0x3f_fff_fff_fff_fff_fff_fff_fffu128;

const PAWNPUT: SideBoardArray = {
    let mut array: SideBoardArray = [0; SIDECOUNT];
    let mut index = 0;
    while index < array.len() {
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

    array
};

const fn get_one_index_array(mut value: u128) -> (IndexSeatArray, usize) {
    let mut array: IndexSeatArray = [0; SEATCOUNT];
    let mut count: usize = 0;
    while value != 0 {
        let index = value.trailing_zeros() as usize;
        array[count] = index;

        value ^= MASK[index];
        count += 1;
    }

    (array, count)
}

// 帅仕根据所处的位置选取可移动位棋盘[index:0-89]
const KINGMOVE: SeatBoardArray = {
    let mut array: SeatBoardArray = [0; SEATCOUNT];
    let (index_array, mut count) = get_one_index_array(KINGPUT[0] | KINGPUT[1]);
    while count > 0 {
        let index = index_array[count - 1];
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

        count -= 1;
    }

    array
};

const ADVISORMOVE: SeatBoardArray = {
    let mut array: SeatBoardArray = [0; SEATCOUNT];
    let (index_array, mut count) = get_one_index_array(ADVISORPUT[0] | ADVISORPUT[1]);
    while count > 0 {
        let index = index_array[count - 1];
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

        count -= 1;
    }

    array
};

// 马相根据憋马腿或田心组成的四个位置状态选取可移动位棋盘[state:0-0XF][index:0-89]
const BISHOPMOVE: LegStateSeatBoardArray = {
    let mut array: LegStateSeatBoardArray = [[0; SEATCOUNT]; LEGSTATECOUNT];
    let mut state = 0;
    while state < LEGSTATECOUNT {
        let mut all_move = [0u128; SEATCOUNT];
        let (index_array, mut count) = get_one_index_array(BISHOPPUT[0] | BISHOPPUT[1]);
        while count > 0 {
            let index = index_array[count - 1];
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

            count -= 1;
        }

        array[state] = all_move;
        state += 1;
    }

    array
};

const KNIGHTMOVE: LegStateSeatBoardArray = {
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
};

// 车炮根据每行和每列的位置状态选取可移动位棋盘[state:0-0x1FF,0X3FF][index:0-89]

// private static readonly List<BigInteger[]> RookRowMove = CreateRookCannonMove(PieceKind.Rook, false);
// private static readonly List<BigInteger[]> RookColMove = CreateRookCannonMove(PieceKind.Rook, true);
// private static readonly List<BigInteger[]> CannonRowMove = CreateRookCannonMove(PieceKind.Cannon, false);
// private static readonly List<BigInteger[]> CannonColMove = CreateRookCannonMove(PieceKind.Cannon, true);

// 兵根据本方处于上或下的二个位置状态选取可移动位棋盘[isBottom:0-1][index:0-89]
const PAWNMOVE: SideSeatBoardArray = {
    let mut array: SideSeatBoardArray = [[0; SEATCOUNT]; SIDECOUNT];
    let mut side = 0;
    while side < SIDECOUNT {
        let mut all_move = [0u128; SEATCOUNT];
        let (index_array, mut count) = get_one_index_array(PAWNPUT[0] | PAWNPUT[1]);
        while count > 0 {
            let index = index_array[count - 1];
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

            count -= 1;
        }

        array[side] = all_move;
        side += 1;
    }

    array
};

pub fn get_board_string(value: u128, is_rotate: bool) -> Vec<String> {
    fn get_rowcol_string(value: u128, col_num: usize) -> String {
        let mut result = String::new();
        for col in 0..col_num {
            result += if (value & (1 << col)) == 0 { "-" } else { "1" };
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
            (value & (mode << offset)) >> offset,
            col_num,
        ));
    }

    return result;
}

pub fn GetBigIntArrayString(
    bigInts: Vec<u128>,
    colNumPerRow: usize,
    showZero: bool,
    is_rotate: bool,
) {
    let row_num = if is_rotate { COLCOUNT } else { ROWCOUNT };
    // int rowNum = isRotate ? Coord.ColCount : Coord.RowCount;
    let length = bigInts.len();
    let colNumPerRow = if length < colNumPerRow {
        length
    } else {
        colNumPerRow
    };
    let mut nullStr = String::from("   ");
    for col in 0..colNumPerRow {
        nullStr += if is_rotate {
            "ABCDEFGHIJ "
        } else {
            "ABCDEFGHI "
        };
    }
    nullStr += "\n";

    // if !showZero
    // {
    //     let mut count = 0;
    //     BigInteger[] nonZeroBigInts = new BigInteger[length];
    //     for (int index = 0; index < length; ++index)
    //     {
    //         if (!bigInts[index].IsZero)
    //             nonZeroBigInts[count++] = bigInts[index];
    //     }
    //     bigInts = nonZeroBigInts;
    //     length = count;
    // }

    // StringBuilder result = new();
    // for (int index = 0; index < length; index += colNumPerRow)
    // {
    //     List<List<string>> resultPerRow = new();
    //     for (int col = 0; col < colNumPerRow && index + col < length; ++col)
    //     {
    //         resultPerRow.Add(GetBigIntString(bigInts[index + col], isRotate));
    //     }

    //     StringBuilder rowResult = new();
    //     rowResult.Append(nullStr);
    //     for (int row = 0; row < rowNum; ++row)
    //     {
    //         rowResult.Append($"{row}: ");
    //         for (int col = 0; col < colNumPerRow && index + col < length; ++col)
    //             rowResult.Append(resultPerRow[col][row]);

    //         rowResult.Append("\n");
    //     }

    //     result.Append(rowResult);
    // }
    // result.Append($"length: {length}\n");

    // return result.ToString();
}
