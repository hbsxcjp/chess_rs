#![allow(dead_code)]
// #![allow(unused_imports)]

// use serde_derive::{Deserialize, Serialize};
// use std::cell::RefCell;
use crate::models::{AspectData, EvaluationData, ZorbistData};
use crate::schema::{aspect, evaluation, zorbist};
use diesel::prelude::*;
use diesel::result::Error;
use diesel::sqlite::SqliteConnection;
use std::collections::HashMap;

use crate::{bit_board, bit_constant, coord, piece};

#[derive(Debug)]
pub struct Evaluation {
    count: usize,
}

// to_index->Evaluation
#[derive(Debug)]
pub struct ToIndex {
    inner: HashMap<usize, Evaluation>,
}

// from_index->IndexEvaluation
#[derive(Debug)]
pub struct Aspect {
    inner: HashMap<usize, ToIndex>,
}

// #[derive(Debug)]
#[derive(Debug)]
pub struct Zorbist {
    inner: HashMap<u64, (u64, Aspect)>,
}

// 后期根据需要扩展
impl Evaluation {
    pub fn from(count: usize) -> Evaluation {
        Evaluation { count }
    }

    pub fn append(&mut self, evaluation: Evaluation) {
        self.count += evaluation.count;
    }

    pub fn to_string(&self) -> String {
        format!("{}", self.count)
    }
}

impl ToIndex {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn from(to_index: usize, evaluation: Evaluation) -> Self {
        let mut index_evaluation = Self::new();
        index_evaluation.inner.insert(to_index, evaluation);

        index_evaluation
    }

    // pub fn from(to_index_counts: Vec<(usize, usize)>) -> Self {
    //     let mut result = Self::new();
    //     for (to_index, count) in to_index_counts {
    //         result
    //             .inner
    //             .borrow_mut()
    //             .insert(to_index, Evaluation::new(count));
    //     }

    //     result
    // }

    pub fn insert(&mut self, to_index: usize, evaluation: Evaluation) {
        if !self.inner.contains_key(&to_index) {
            self.inner.insert(to_index, evaluation);
        } else {
            self.inner.get_mut(&to_index).unwrap().append(evaluation);
        }
    }

    pub fn append(&mut self, other_index_evaluation: Self) {
        for (to_index, evaluation) in other_index_evaluation.inner {
            self.insert(to_index, evaluation);
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

impl Aspect {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn from(from_index: usize, to_index: ToIndex) -> Self {
        let mut aspect = Self::new();
        aspect.inner.insert(from_index, to_index);

        aspect
    }

    pub fn insert(&mut self, from_index: usize, index_evaluation: ToIndex) {
        if self.inner.contains_key(&from_index) {
            self.inner
                .get_mut(&from_index)
                .unwrap()
                .append(index_evaluation);
        } else {
            self.inner.insert(from_index, index_evaluation);
        }
    }

    pub fn append(&mut self, other: Self) {
        for (from_index, to_index) in other.inner {
            self.insert(from_index, to_index);
        }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (from_index, index_evaluation) in self.inner.iter() {
            let coord = coord::Coord::from_index(*from_index).unwrap();
            result.push_str(&format!(
                "{}=>{}",
                coord.to_string(coord::RecordType::Txt),
                index_evaluation.to_string()
            ));
        }
        result.push_str(&format!("aspect_evaluation.len:{}\n", self.inner.len()));

        result
    }
}

impl Zorbist {
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
        }
    }

    pub fn from(key: u64, lock: u64, aspect_evaluation: Aspect) -> Self {
        let mut zorbist = Self::new();
        zorbist.insert(key, lock, aspect_evaluation);

        zorbist
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn insert(&mut self, id: u64, lock: u64, aspect: Aspect) {
        match self.get_mut_aspect_evaluation(id, lock) {
            Some(old_aspect) => old_aspect.append(aspect),
            None => {
                self.inner.insert(id, (lock, aspect));
            }
        }
    }

    pub fn append(&mut self, other_zorbist: Self) {
        for (key, (lock, aspect_evaluation)) in other_zorbist.inner {
            self.insert(key, lock, aspect_evaluation);
        }
    }

    pub fn get_aspect_evaluation(
        &self,
        bit_board: &bit_board::BitBoard,
        color: piece::Color,
    ) -> Option<&Aspect> {
        let (key, lock) = bit_board.get_key_lock(color);
        let real_key = self.get_real_key(key, lock)?;

        self.inner
            .get(&real_key)
            .map(|(_, aspect_evaluation)| aspect_evaluation)
    }

    fn get_mut_aspect_evaluation(&mut self, key: u64, lock: u64) -> Option<&mut Aspect> {
        let real_key = self.get_real_key(key, lock)?;

        self.inner
            .get_mut(&real_key)
            .map(|(_, aspect_evaluation)| aspect_evaluation)
    }

    fn get_real_key(&self, key: u64, lock: u64) -> Option<u64> {
        let mut real_key = key;
        if !self.inner.contains_key(&key) {
            return None;
        }

        for index in 0..bit_constant::COLLIDEZOBRISTKEY.len() {
            if self.inner.contains_key(&real_key) && lock == self.inner.get(&real_key).unwrap().0 {
                break;
            }

            assert!(
                false,
                "Key:({key:016x})->RealKey:({real_key:016x})'s Lock({lock:016x}) is not find!\n"
            );

            real_key ^= bit_constant::COLLIDEZOBRISTKEY[index];
        }

        // assert!(
        //     self.inner.contains_key(&real_key),
        //     "Key:({key:016x})->RealKey:({real_key:016x})'s Lock({lock:016x}) is not find!\n"
        // );

        Some(real_key)
    }

    pub fn from_db(conn: &mut SqliteConnection) -> Result<Self, Error> {
        let eva_asp_zor: Vec<(EvaluationData, AspectData, ZorbistData)> = evaluation::table
            .inner_join(aspect::table.inner_join(zorbist::table))
            .select((
                EvaluationData::as_select(),
                AspectData::as_select(),
                ZorbistData::as_select(),
            ))
            .load::<(EvaluationData, AspectData, ZorbistData)>(conn)?;

        let mut zorbist = Self::new();
        for (eva, asp, zor) in eva_asp_zor {
            zorbist.insert(
                zor.id as u64,
                zor.lock as u64,
                Aspect::from(
                    asp.from_index as usize,
                    ToIndex::from(eva.to_index as usize, Evaluation::from(eva.count as usize)),
                ),
            );
        }

        Ok(zorbist)
    }

    // 每次全新保存数据
    pub fn save_db(&self, conn: &mut SqliteConnection) -> Result<(usize, usize, usize), Error> {
        let mut zorbist_datas = vec![];
        let mut aspect_datas = vec![];
        let mut evaluation_datas = vec![];
        let mut aspect_id = 0;
        for (id, (lock, aspect)) in self.inner.iter() {
            let id = *id as i64;
            let lock = *lock as i64;
            zorbist_datas.push(ZorbistData { id, lock });
            for (from_index, to_index_eval) in aspect.inner.iter() {
                let from_index = *from_index as i32;
                aspect_id += 1;
                aspect_datas.push(AspectData {
                    id: aspect_id,
                    from_index,
                    zorbist_id: id,
                });
                for (to_index, eval) in to_index_eval.inner.iter() {
                    let to_index = *to_index as i32;
                    evaluation_datas.push(EvaluationData {
                        to_index,
                        count: eval.count as i32,
                        aspect_id,
                    });
                }
            }
        }

        // Sqlite3 "PRAGMA foreign_keys = ON"
        let _ = diesel::delete(zorbist::table).execute(conn)?;
        let zor_count = diesel::insert_into(zorbist::table)
            .values(zorbist_datas)
            .execute(conn)?;
        let asp_count = diesel::insert_into(aspect::table)
            .values(aspect_datas)
            .execute(conn)?;
        let eva_count = diesel::insert_into(evaluation::table)
            .values(evaluation_datas)
            .execute(conn)?;

        Ok((zor_count, asp_count, eva_count))
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
    // use super::*;
    use crate::manual;
    use crate::models;

    #[test]
    // #[ignore = "从文件提取zorbist后存入数据库。"]
    fn test_evaluation() {
        let filename_manuals = crate::common::get_filename_manuals();
        let manuals = filename_manuals
            .into_iter()
            .map(|(_, _, manual)| manual)
            .collect::<Vec<manual::Manual>>();
        let zorbist = manual::get_zorbist_manuals(manuals);

        // let mut conn = database::get_connection();
        // let zorbist = database::get_zorbist(&mut conn);
        // println!("zorbist: {}", zorbist.len());
        let mut conn = models::get_conn(&models::get_pool());
        let result = zorbist.save_db(&mut conn).expect("Save Err.");
        println!("evaluation zor_asp_eva: {:?}", result);

        let result = zorbist.to_string();
        std::fs::write(format!("tests/output/zobrist_file.txt"), result).expect("Write Err.");

        // let json_file_name = "tests/output/serde_json.txt";
        // let result = serde_json::to_string(&zorbist).unwrap();
        // std::fs::write(json_file_name, result).expect("Write Err.");

        // serde_json
        // let vec_u8 = std::fs::read(json_file_name).unwrap();
        // let zorbist_eval: ZorbistEvaluation =
        //     serde_json::from_str(&String::from_utf8(vec_u8).unwrap()).unwrap();
        // println!("zorbist_eval: {}", zorbist_eval.len());
        // let result = zorbist_eval.to_string();
        // std::fs::write(format!("tests/output/zobrist_eval.txt"), result).expect("Write Err.");
    }

    #[test]
    #[ignore = "从数据库提取zorbist。"]
    fn test_evaluation_db() {}
}
