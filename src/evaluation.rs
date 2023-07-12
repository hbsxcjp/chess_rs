#![allow(dead_code)]
// #![allow(unused_imports)]

use std::cell::RefCell;
use std::collections::HashMap;

use crate::{bit_board, bit_constant, coord, piece};

#[derive(Debug)]
pub struct Evaluation {
    is_killed: bool,

    eat_kind: piece::Kind,
    count: usize,
}

// to_index->Evaluation
pub struct IndexEvaluation {
    inner: HashMap<usize, Evaluation>,
}

// from_index->IndexEvaluation
pub struct AspectEvaluation {
    inner: RefCell<HashMap<usize, IndexEvaluation>>,
}

// #[derive(Debug)]
pub struct ZorbistAspectEvaluation {
    inner: HashMap<u64, (u64, AspectEvaluation)>,
}

// 后期根据需要扩展
impl Evaluation {
    pub fn new(is_killed: bool, eat_kind: piece::Kind, count: usize) -> Evaluation {
        Evaluation {
            is_killed,
            eat_kind,
            count,
        }
    }

    pub fn increase(&mut self) {
        self.count += 1;
    }

    pub fn to_string(&self) -> String {
        format!("{},{:?},{}", self.is_killed, self.eat_kind, self.count)
    }
}

impl IndexEvaluation {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, to_index: usize, evaluation: Evaluation) {
        if !self.inner.contains_key(&to_index) {
            self.inner.insert(to_index, evaluation);
            return;
        }

        self.inner.get_mut(&to_index).unwrap().increase();
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
        aspect_evaluation.insert(from_index, IndexEvaluation::new());

        aspect_evaluation
    }

    pub fn insert_evaluation(&self, from_index: usize, to_index: usize, evaluation: Evaluation) {
        if !self.inner.borrow().contains_key(&from_index) {
            self.insert(from_index, IndexEvaluation::new());
        }

        self.inner
            .borrow_mut()
            .get_mut(&from_index)
            .unwrap()
            .insert(to_index, evaluation);
    }

    pub fn insert(&self, from_index: usize, index_evaluation: IndexEvaluation) {
        self.inner.borrow_mut().insert(from_index, index_evaluation);
    }

    pub fn append(&self, other_aspect_evaluation: Self) {
        for (from_index, index_evaluation) in other_aspect_evaluation.inner.into_inner() {
            self.insert(from_index, index_evaluation);
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

impl ZorbistAspectEvaluation {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: u64, lock: u64, aspect_evaluation: AspectEvaluation) {
        match self.get_aspect_evaluation(key, lock) {
            Some(old_aspect_evaluation) => old_aspect_evaluation.append(aspect_evaluation),
            None => {
                self.inner.insert(key, (lock, aspect_evaluation));
            }
        }
    }

    pub fn get_aspect_evaluation_from_bit_board(
        &self,
        bit_board: bit_board::BitBoard,
        color: piece::Color,
    ) -> Option<&AspectEvaluation> {
        self.get_aspect_evaluation(bit_board.get_key(color), bit_board.get_lock(color))
    }

    pub fn append(&mut self, other_zorbist_aspect_evaluation: Self) {
        for (key, (lock, aspect_evaluation)) in other_zorbist_aspect_evaluation.inner {
            self.insert(key, lock, aspect_evaluation);
        }
    }

    fn get_aspect_evaluation(&self, key: u64, lock: u64) -> Option<&AspectEvaluation> {
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

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (key, (lock, aspect_evaluation)) in self.inner.iter() {
            result.push_str(&format!("key:  {:016x}\nlock: {:016x}\n", key, lock));
            result.push_str(&aspect_evaluation.to_string());
        }

        result.push_str(&format!(
            "zorbist_aspect_evaluation.len: {}\n",
            self.inner.len()
        ));

        result
    }
}
