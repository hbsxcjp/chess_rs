#![allow(dead_code)]
// #![allow(unused_imports)]

use serde_derive::{Deserialize, Serialize};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::{bit_constant, coord};

#[derive(Serialize, Deserialize, Debug)]
pub struct Evaluation {
    count: usize,
}

// to_index->Evaluation
#[derive(Serialize, Deserialize, Debug)]
pub struct IndexEvaluation {
    inner: HashMap<usize, Evaluation>,
}

// from_index->IndexEvaluation
#[derive(Serialize, Deserialize, Debug)]
pub struct AspectEvaluation {
    inner: RefCell<HashMap<usize, IndexEvaluation>>,
}

// #[derive(Debug)]
#[derive(Serialize, Deserialize, Debug)]
pub struct ZorbistEvaluation {
    inner: HashMap<u64, (u64, AspectEvaluation)>,
}

// 后期根据需要扩展
impl Evaluation {
    pub fn new(count: usize) -> Evaluation {
        Evaluation { count }
    }

    pub fn exists_operate(&mut self) {
        self.count += 1;
    }

    pub fn to_string(&self) -> String {
        format!("{}", self.count)
    }
}

impl IndexEvaluation {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn from(to_index_counts: Vec<(usize, usize)>) -> Self {
        let mut result = Self::new();
        for (to_index, count) in to_index_counts {
            result.inner.insert(to_index, Evaluation::new(count));
        }

        result
    }

    pub fn insert(&mut self, to_index: usize, evaluation: Evaluation) {
        if !self.inner.contains_key(&to_index) {
            self.inner.insert(to_index, evaluation);
        } else {
            self.inner.get_mut(&to_index).unwrap().exists_operate();
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (to_index, evaluation) in self.inner.iter() {
            let coord = coord::Coord::from_index(*to_index).unwrap();
            result.push_str(&format!(
                "[{} {}] ",
                coord.to_string(coord::RecordType::Txt),
                evaluation.to_string()
            ));
        }
        result.push_str(&format!("【{}】\n", self.inner.len()));

        result
    }
}

impl AspectEvaluation {
    pub fn new() -> Self {
        Self {
            inner: RefCell::new(HashMap::new()),
        }
    }

    pub fn from(from_index: usize) -> Self {
        let aspect_evaluation = Self::new();
        aspect_evaluation
            .inner
            .borrow_mut()
            .insert(from_index, IndexEvaluation::new());

        aspect_evaluation
    }

    pub fn from_values(from_index: usize, to_index: usize, count: usize) -> Self {
        let aspect_evaluation = Self::from(from_index);
        aspect_evaluation.insert_evaluation(from_index, to_index, Evaluation { count });

        aspect_evaluation
    }

    pub fn insert_evaluation(&self, from_index: usize, to_index: usize, evaluation: Evaluation) {
        if !self.inner.borrow().contains_key(&from_index) {
            self.inner
                .borrow_mut()
                .insert(from_index, IndexEvaluation::new());
        }

        self.inner
            .borrow_mut()
            .get_mut(&from_index)
            .unwrap()
            .insert(to_index, evaluation);
    }

    pub fn append(&self, other_aspect_evaluation: Self) {
        for (from_index, index_evaluation) in other_aspect_evaluation.inner.into_inner() {
            if index_evaluation.inner.is_empty() {
                self.inner.borrow_mut().insert(from_index, index_evaluation);
            } else {
                for (to_index, evaluation) in index_evaluation.inner {
                    self.insert_evaluation(from_index, to_index, evaluation);
                }
            }
        }
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (from_index, index_evaluation) in self.inner.borrow().iter() {
            let coord = coord::Coord::from_index(*from_index).unwrap();
            result.push_str(&format!(
                "{}=>{}",
                coord.to_string(coord::RecordType::Txt),
                index_evaluation.to_string()
            ));
        }
        result.push_str(&format!(
            "aspect_evaluation.len:{}\n",
            self.inner.borrow().len()
        ));

        result
    }
}

impl ZorbistEvaluation {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn from(key: u64, lock: u64, aspect_evaluation: AspectEvaluation) -> Self {
        let mut zorbist_evaluation = Self::new();
        zorbist_evaluation.insert_kla(key, lock, aspect_evaluation);

        zorbist_evaluation
    }

    pub fn insert_values(&mut self, values: (u64, u64, usize, usize, usize)) {
        let (key, lock, from_index, to_index, count) = values;
        self.insert_kla(
            key,
            lock,
            AspectEvaluation::from_values(from_index, to_index, count),
        );
    }

    pub fn insert_kla(&mut self, key: u64, lock: u64, aspect_evaluation: AspectEvaluation) {
        match self.get_aspect_evaluation(key, lock) {
            Some(old_aspect_evaluation) => old_aspect_evaluation.append(aspect_evaluation),
            None => {
                self.inner.insert(key, (lock, aspect_evaluation));
            }
        }
    }

    pub fn append(&mut self, other_zorbist_evaluation: Self) {
        for (key, (lock, aspect_evaluation)) in other_zorbist_evaluation.inner {
            self.insert_kla(key, lock, aspect_evaluation);
        }
    }

    pub fn get_aspect_evaluation(&self, key: u64, lock: u64) -> Option<&AspectEvaluation> {
        let mut real_key = key;
        if !self.inner.contains_key(&key) {
            return None;
        }

        for index in 0..bit_constant::COLLIDEZOBRISTKEY.len() {
            if self.inner.contains_key(&real_key) && lock == self.inner.get(&real_key).unwrap().0 {
                break;
            }

            real_key ^= bit_constant::COLLIDEZOBRISTKEY[index];
        }

        assert!(
            self.inner.contains_key(&real_key),
            "Key:({key:016x})->RealKey:({real_key:016x})'s Lock({lock:016x}) is not find!\n"
        );
        self.inner
            .get(&real_key)
            .map(|(_, aspect_evaluation)| aspect_evaluation)
    }

    pub fn get_data_values(&self) -> Vec<(u64, u64, usize, usize, usize)> {
        let mut result = vec![];
        for (key, lock_aspect_eval) in &self.inner {
            let (lock, aspect_eval) = lock_aspect_eval;
            for (from_index, index_eval) in aspect_eval.inner.borrow().iter() {
                for (to_index, eval) in &index_eval.inner {
                    result.push((*key, *lock, *from_index, *to_index, eval.count));
                }
            }
        }

        result
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (key, (lock, aspect_evaluation)) in self.inner.iter() {
            result.push_str(&format!("key:  {} lock: {}\n", key, lock));
            result.push_str(&aspect_evaluation.to_string());
        }

        result.push_str(&format!(
            "zorbist_aspect_evaluation.len: {}\n",
            self.inner.len()
        ));

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common;

    #[test]
    fn test_evaluation() {
        let filename_manuals = crate::common::get_filename_manuals();
        let zorbist_evaluation = common::get_some_zorbist_evaluation(&filename_manuals);

        let result = zorbist_evaluation.to_string();
        std::fs::write(format!("tests/output/zobrist_evaluation.txt"), result).expect("Write Err.");

        let json_file_name = "tests/output/serde_json.txt";
        let result = serde_json::to_string(&zorbist_evaluation).unwrap();
        std::fs::write(json_file_name, result).expect("Write Err.");

        // serde_json
        let vec_u8 = std::fs::read(json_file_name).unwrap();
        let zorbist_eval: ZorbistEvaluation =
            serde_json::from_str(&String::from_utf8(vec_u8).unwrap()).unwrap();
        let result = zorbist_eval.to_string();
        std::fs::write(format!("tests/output/zobrist_eval.txt"), result).expect("Write Err.");
    }
}
