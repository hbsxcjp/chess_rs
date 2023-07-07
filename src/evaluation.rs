#![allow(dead_code)]
// #![allow(unused_imports)]

use std::collections::HashMap;

// use crate::bit_board;
use crate::coord;
// use crate::piece;

#[derive(Debug)]
pub struct Evaluation {
    is_killed: bool,

    count: usize,
}

// to_index->Evaluation
pub struct IndexEvaluation {
    index_evaluation: HashMap<usize, Evaluation>,
}

// from_index->IndexEvaluation
pub struct AspectEvaluation {
    aspect_evaluation: HashMap<usize, IndexEvaluation>,
}

// #[derive(Debug)]
pub struct ZorbistAspectEvaluation {
    key_lock_aspect_evaluation: HashMap<u64, (u64, AspectEvaluation)>,
}

// 后期根据需要扩展
impl Evaluation {
    pub fn new(is_killed: bool, count: usize) -> Evaluation {
        Evaluation { is_killed, count }
    }

    pub fn to_string(&self) -> String {
        format!("{}-{} ", self.is_killed, self.count)
    }
}

impl IndexEvaluation {
    pub fn new() -> Self {
        Self {
            index_evaluation: HashMap::new(),
        }
    }

    // pub fn from(to_index: usize, is_killed: bool, count: usize) -> Self {
    //     let mut index_evaluation = Self::new();
    //     index_evaluation.insert(to_index, Evaluation::new(is_killed, count));

    //     index_evaluation
    // }

    pub fn insert(&mut self, to_index: usize, evaluation: Evaluation) {
        self.index_evaluation.insert(to_index, evaluation);
    }

    // pub fn append(&mut self, other_index_evaluation: Self) {
    //     for (to_index, evaluation) in other_index_evaluation.index_evaluation {
    //         self.index_evaluation.insert(to_index, evaluation);
    //     }
    // }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (to_index, evaluation) in self.index_evaluation.iter() {
            let coord = coord::Coord::from_index(*to_index).unwrap();
            result.push_str(&format!(
                "{}-{}",
                coord.to_string(coord::RecordType::Txt),
                evaluation.to_string()
            ));
        }
        result.push_str(&format!("【{}】\n", self.index_evaluation.len()));

        result
    }
}

impl AspectEvaluation {
    pub fn new() -> Self {
        Self {
            aspect_evaluation: HashMap::new(),
        }
    }

    pub fn from(from_index: usize) -> Self {
        let mut aspect_evaluation = Self::new();
        aspect_evaluation.insert(from_index, IndexEvaluation::new());

        aspect_evaluation
    }

    pub fn insert_evaluation(
        &mut self,
        from_index: usize,
        to_index: usize,
        evaluation: Evaluation,
    ) {
        if !self.aspect_evaluation.contains_key(&from_index) {
            self.insert(from_index, IndexEvaluation::new());
        }

        self.aspect_evaluation
            .get_mut(&from_index)
            .unwrap()
            .insert(to_index, evaluation);
    }

    pub fn insert(&mut self, from_index: usize, index_evaluation: IndexEvaluation) {
        self.aspect_evaluation.insert(from_index, index_evaluation);
    }

    pub fn append(&mut self, other_aspect_evaluation: Self) {
        for (from_index, index_evaluation) in other_aspect_evaluation.aspect_evaluation {
            self.aspect_evaluation.insert(from_index, index_evaluation);
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (from_index, index_evaluation) in self.aspect_evaluation.iter() {
            let coord = coord::Coord::from_index(*from_index).unwrap();
            result.push_str(&format!(
                "{} => {}",
                coord.to_string(coord::RecordType::Txt),
                index_evaluation.to_string()
            ));
        }
        result.push_str(&format!(
            "aspect_evaluation.len: {}\n",
            self.aspect_evaluation.len()
        ));

        result
    }
}

impl ZorbistAspectEvaluation {
    pub fn new(
        key: u64,
        lock: u64,
        aspect_evaluation: AspectEvaluation,
    ) -> ZorbistAspectEvaluation {
        let mut zorbist_aspect_evaluation = ZorbistAspectEvaluation {
            key_lock_aspect_evaluation: HashMap::new(),
        };

        zorbist_aspect_evaluation
            .key_lock_aspect_evaluation
            .insert(key, (lock, aspect_evaluation));

        zorbist_aspect_evaluation
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (key, (lock, aspect_evaluation)) in self.key_lock_aspect_evaluation.iter() {
            result.push_str(&format!("key:  {:016x}\nlock: {:016x}\n", key, lock));
            result.push_str(&aspect_evaluation.to_string());
        }

        result.push_str(&format!(
            "zorbist_aspect_evaluation.len: {}\n",
            self.key_lock_aspect_evaluation.len()
        ));

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
