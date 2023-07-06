#![allow(dead_code)]
// #![allow(unused_imports)]

use std::collections::HashMap;

use crate::bit_board;
// use crate::bit_constant;
use crate::coord;
use crate::piece;
// use crate::piece; //::{self, COLORCOUNT, KINDCOUNT};

#[derive(Debug)]
pub struct Evaluation {
    to_index: usize,

    score: i32,
    count: i32,
}

// #[derive(Debug)]
// pub struct PerPossible {
//     pub from_index: usize,

//     possibles: Vec<Evaluation>,
// }

// #[derive(Debug)]
// pub struct LockAllPossible {
//     lock: u64,

//     all_possible: Vec<PerPossible>,
// }

// to_index->possible
pub type IndexEvaluation = HashMap<usize, Evaluation>;

// from_index->to_index->possible
pub type AspectEvaluation = HashMap<usize, IndexEvaluation>;

#[derive(Debug)]
pub struct HistoryEvaluation {
    // zobrist->from_index->to_index->possible
    zobrist_all_possible: HashMap<u64, AspectEvaluation>,
}

// 后期根据需要扩展
impl Evaluation {
    pub fn new(to_index: usize, score: i32, count: i32) -> Evaluation {
        Evaluation {
            to_index,
            score,
            count,
        }
    }

    pub fn to_string(&self) -> String {
        let coord = coord::Coord::from_index(self.to_index).unwrap();
        format!(
            "{}-{}-{} ",
            coord.to_string(coord::RecordType::Txt),
            self.score,
            self.count
        )
    }
}

// impl PerPossible {
//     pub fn new(from_index: usize) -> PerPossible {
//         PerPossible {
//             from_index,
//             possibles: Vec::new(),
//         }
//     }

//     pub fn add(&mut self, to_index: usize, score: i32, count: i32) {
//         self.possibles.push(Evaluation::new(to_index, score, count));
//     }

//     pub fn to_string(&self) -> String {
//         let coord = coord::Coord::from_index(self.from_index).unwrap();
//         let mut result = format!("{} => ", coord.to_string(coord::RecordType::Txt));
//         for possible in self.possibles.iter() {
//             result.push_str(&possible.to_string());
//         }
//         result.push_str(&format!("【{}】\n", self.possibles.len()));

//         result
//     }
// }

// impl LockAllPossible {
//     pub fn from(bit_board: &mut bit_board::BitBoard, color: piece::Color) -> LockAllPossible {
//         LockAllPossible {
//             lock: bit_board.get_lock(color),
//             all_possible: bit_board.get_possibles_from_color(color),
//         }
//     }

//     pub fn to_string(&self) -> String {
//         let mut result = String::new();
//         for per_possible in self.all_possible.iter() {
//             result.push_str(&per_possible.to_string());
//         }
//         result.push_str(&format!("count: {}\n", self.all_possible.len()));

//         result
//     }
// }

impl HistoryEvaluation {
    pub fn new() -> HistoryEvaluation {
        HistoryEvaluation {
            zobrist_all_possible: HashMap::new(),
        }
    }

    pub fn from(bit_board: &mut bit_board::BitBoard, color: piece::Color) -> HistoryEvaluation {
        let mut history_evaluation = HistoryEvaluation::new();
        history_evaluation.zobrist_all_possible.insert(
            bit_board.get_key(color),
            bit_board.get_aspect_evaluation_from_color(color),
        );

        history_evaluation
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (_key, aspect_evaluation) in self.zobrist_all_possible.iter() {
            for (from_index, index_evaluation) in aspect_evaluation.iter() {
                let coord = coord::Coord::from_index(*from_index).unwrap();
                result.push_str(&format!("{} => ", coord.to_string(coord::RecordType::Txt)));
                for (_to_index, evaluation) in index_evaluation.iter() {
                    result.push_str(&evaluation.to_string());
                }
                result.push_str(&format!("【{}】\n", index_evaluation.len()));
            }
            result.push_str(&format!("count: {}\n", aspect_evaluation.len()));
        }

        // result.push_str(&format!("count: {}\n", self.zobrist_all_possible.len()));

        result
    }

    // pub fn get_move_possible(&self, mut key: u64, lock: u64) -> Option<&Vec<PerPossible>> {
    //     for index in 0..bit_constant::COLLIDEZOBRISTKEY.len() {
    //         if let Some(lock_all_possible) = self.key_lock_all_possible.get(&key) {
    //             if lock_all_possible.lock == lock {
    //                 return Some(&lock_all_possible.all_possible);
    //             }
    //         }

    //         key ^= bit_constant::COLLIDEZOBRISTKEY[index];
    //         assert!(false, "hashlock is not same! index:{index}\n");
    //     }

    //     None
    // }

    // pub fn to_string(&self) -> String {
    //     let mut result = String::new();
    //     for (key, lock_all_possible) in self.key_lock_all_possible.iter() {
    //         result.push_str(&format!(
    //             "hashkey:{:016x}\nmove_possible:\n{}\n",
    //             key,
    //             lock_all_possible.to_string()
    //         ));
    //     }
    //     result.push_str(&format!(
    //         "history_len:【{}】\n",
    //         self.key_lock_all_possible.len()
    //     ));

    //     result
    // }
}
