#![allow(dead_code)]
// #![allow(unused_imports)]

// use serde_derive::{Deserialize, Serialize};
// use std::cell::RefCell;
// use rayon::vec;
use crate::models::ManualInfo;
use crate::{bit_board, bit_constant, coord};
use diesel::result::Error;
use diesel::sqlite::SqliteConnection;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Display, Formatter}; //coord,

#[derive(Clone, Copy)]
pub struct Evaluation {
    count: usize,
}

// #[derive(Debug)]
pub struct ToIndex {
    to: usize,
    eval: Evaluation,
}

// #[derive(Debug)]
pub struct FromIndex {
    from: usize,
    to_indexs: Vec<ToIndex>,
}

// #[derive(Debug)]
pub struct Aspect {
    lock: u64,
    from_indexs: Vec<FromIndex>,
}

// #[derive(Debug)]
pub struct Zorbist {
    key_aspects: HashMap<u64, Aspect>,
}

pub struct FromToIndex {
    from: usize,
    to: usize,
    eval: Evaluation,
}

impl Display for Evaluation {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.count)
    }
}

impl Display for ToIndex {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let coord = coord::Coord::from_index(self.to).unwrap();
        write!(f, "{}={}", coord, self.eval)
    }
}

impl Display for FromIndex {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let coord = coord::Coord::from_index(self.from).unwrap();
        let _ = write!(f, "{}->[", coord)?;
        for to_index in &self.to_indexs {
            let _ = write!(f, "{} ", to_index)?;
        }
        let _ = write!(f, "]【{}】\n", self.to_indexs.len())?;

        Ok(())
    }
}

impl Display for FromToIndex {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let from_coord = coord::Coord::from_index(self.from).unwrap();
        let to_coord = coord::Coord::from_index(self.to).unwrap();

        write!(f, "{}->{}={}", from_coord, to_coord, self.eval)
    }
}

impl Display for Aspect {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        let _ = write!(f, "lock:{:016X}\n", self.lock)?;
        for from_index in &self.from_indexs {
            let _ = write!(f, "{}", from_index)?;
        }
        let _ = write!(f, "count: 【{}】\n", self.from_indexs.len())?;

        Ok(())
    }
}

impl Display for Zorbist {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        for (key, aspect) in &self.key_aspects {
            let _ = write!(f, "key:{:016X} {}", key, aspect)?;
        }
        let _ = write!(f, "zorbist count: 【{}】\n\n", self.key_aspects.len())?;

        Ok(())
    }
}

impl Ord for Evaluation {
    fn cmp(&self, other: &Self) -> Ordering {
        other.count.cmp(&self.count)
    }
}

impl PartialOrd for Evaluation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Evaluation {
    fn eq(&self, other: &Self) -> bool {
        self.count == other.count
    }
}

impl Eq for Evaluation {}

impl Ord for ToIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.to.cmp(&other.to)
    }
}

impl PartialOrd for ToIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for ToIndex {
    fn eq(&self, other: &Self) -> bool {
        self.to == other.to
    }
}

impl Eq for ToIndex {}

impl Ord for FromIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.from.cmp(&other.from)
    }
}

impl PartialOrd for FromIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FromIndex {
    fn eq(&self, other: &Self) -> bool {
        self.from == other.from
    }
}

impl Eq for FromIndex {}

impl Ord for FromToIndex {
    fn cmp(&self, other: &Self) -> Ordering {
        self.eval.cmp(&other.eval)
    }
}

impl PartialOrd for FromToIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for FromToIndex {
    fn eq(&self, other: &Self) -> bool {
        self.eval == other.eval
    }
}

impl Eq for FromToIndex {}

// 后期根据需要扩展
impl Evaluation {
    pub fn from(count: usize) -> Self {
        Self { count }
    }

    pub fn insert(&mut self, other: Self) {
        self.count += other.count;
    }
}

impl ToIndex {
    pub fn from(to: usize, eval: Evaluation) -> Self {
        Self { to, eval }
    }

    pub fn insert(&mut self, eval: Evaluation) {
        self.eval.insert(eval);
    }
}

impl FromIndex {
    pub fn from(from: usize, to_indexs: Vec<ToIndex>) -> Self {
        Self { from, to_indexs }
    }

    pub fn insert(&mut self, to_index: ToIndex) {
        match self.to_indexs.binary_search(&to_index) {
            Ok(index) => self.to_indexs[index].insert(to_index.eval),
            Err(index) => self.to_indexs.insert(index, to_index),
        }
    }
}

impl FromToIndex {
    pub fn from(from: usize, to_index: &ToIndex) -> Self {
        Self {
            from,
            to: to_index.to,
            eval: to_index.eval,
        }
    }
}

impl Aspect {
    pub fn new(lock: u64) -> Self {
        Self {
            lock,
            from_indexs: vec![],
        }
    }

    pub fn from(lock: u64, from: usize, to: usize, eval: Evaluation) -> Self {
        Self {
            lock,
            from_indexs: vec![FromIndex::from(from, vec![ToIndex::from(to, eval)])],
        }
    }

    pub fn insert(&mut self, from_index: FromIndex) {
        match self.from_indexs.binary_search(&from_index) {
            Ok(index) => {
                let old_from_index = &mut self.from_indexs[index];
                for to_index in from_index.to_indexs {
                    old_from_index.insert(to_index);
                }
            }
            Err(index) => self.from_indexs.insert(index, from_index),
        }
    }

    pub fn get_from_to_indexs(&self) -> Vec<FromToIndex> {
        let mut result = vec![];
        for from_index in &self.from_indexs {
            for to_index in &from_index.to_indexs {
                let from_to_index = FromToIndex::from(from_index.from, to_index);
                let index = match result.binary_search(&from_to_index) {
                    Ok(index) => index,
                    Err(index) => index,
                };
                result.insert(index, from_to_index);
            }
        }

        result
    }
}

impl Zorbist {
    pub fn new() -> Self {
        Self {
            key_aspects: HashMap::new(),
        }
    }

    pub fn from(key: u64, aspect: Aspect) -> Self {
        let mut result = Self::new();
        result.insert(key, aspect);

        result
    }

    pub fn insert(&mut self, key: u64, aspect: Aspect) {
        match self.get_mut_aspect(key, aspect.lock) {
            Some(old_aspect) => {
                for from_index in aspect.from_indexs {
                    old_aspect.insert(from_index);
                }
            }
            None => {
                self.key_aspects.insert(key, aspect);
            }
        }
    }

    pub fn append(&mut self, other: Self) {
        for (key, aspect) in other.key_aspects {
            self.insert(key, aspect);
        }
    }

    pub fn get_mut_aspect(&mut self, mut key: u64, lock: u64) -> Option<&mut Aspect> {
        for index in 0..bit_constant::COLLIDEZOBRISTKEY.len() {
            if let Some(aspect) = self.key_aspects.get(&key) {
                if lock == aspect.lock {
                    return self.key_aspects.get_mut(&key);
                }
            } else {
                break;
            }

            assert!(false, "Key:({key:016x})'s Lock({lock:016x}) is not find!\n");
            key ^= bit_constant::COLLIDEZOBRISTKEY[index];
        }

        None
    }

    pub fn from_db(conn: &mut SqliteConnection) -> Result<Self, Error> {
        let mut zorbist = Zorbist::new();
        let bit_board = bit_board::BitBoard::new();
        for rowcols in ManualInfo::get_rowcols(conn)? {
            if let Some(rowcols) = rowcols {
                bit_board.clone().insert_to_zorbist(&mut zorbist, rowcols);
            }
        }

        Ok(zorbist)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::manual;
    use crate::models;

    #[test]
    // #[ignore = "从文件提取zorbist后存入数据库"]
    fn test_eval_from_file() {
        let filename_manuals = crate::common::get_filename_manuals();
        let manuals = filename_manuals
            .into_iter()
            .map(|(_, _, manual)| manual)
            .collect::<Vec<manual::Manual>>();
        let zorbist = manual::get_zorbist_manuals(manuals);
        let result = format!("{}", zorbist);
        std::fs::write(format!("tests/output/zobrist_file.txt"), result).expect("Write Err.");
    }

    #[test]
    #[ignore = "从数据库提取manuals后转换成zorbist. (4.25s-4.68s)"]
    fn test_eval_from_db() {
        let conn = &mut models::get_conn();
        let zorbist = Zorbist::from_db(conn).unwrap();
        println!("From_db_manuals: {}", (zorbist.key_aspects.len()));

        let mut result = String::new();
        for (key, aspect) in &zorbist.key_aspects {
            if aspect.from_indexs.len() > 3 {
                result.push_str(&format!("key:{:016X} lock:{:016X}\n", key, aspect.lock));
                let from_to_indexs = aspect.get_from_to_indexs();
                for from_to_index in &from_to_indexs {
                    result.push_str(&format!("{}\n", from_to_index));
                }
                result.push_str(&format!("count: 【{}】\n\n", from_to_indexs.len()));
            }
        }
        std::fs::write(format!("tests/output/zobrist_aspects.txt"), result).expect("Write Err.");

        // let result = format!("{}", zorbist);
        // std::fs::write(format!("tests/output/zobrist_db.txt"), result).expect("Write Err.");
    }
}
