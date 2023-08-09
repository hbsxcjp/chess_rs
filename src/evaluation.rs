#![allow(dead_code)]
// #![allow(unused_imports)]

// use serde_derive::{Deserialize, Serialize};
// use std::cell::RefCell;
use crate::models::{self, AspectData, EvaluationData, ManualInfo, ZorbistData};
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

    pub fn from(id: u64, lock: u64, aspect_evaluation: Aspect) -> Self {
        let mut zorbist = Self::new();
        zorbist.insert(id, lock, aspect_evaluation);

        zorbist
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    fn insert(&mut self, id: u64, lock: u64, aspect: Aspect) {
        match self.get_mut_aspect_evaluation(id, lock) {
            Some(old_aspect) => old_aspect.append(aspect),
            None => {
                self.inner.insert(id, (lock, aspect));
            }
        }
    }

    pub fn append(&mut self, other_zorbist: Self) {
        for (id, (lock, aspect_evaluation)) in other_zorbist.inner {
            self.insert(id, lock, aspect_evaluation);
        }
    }

    pub fn get_aspect_evaluation(
        &self,
        bit_board: &bit_board::BitBoard,
        color: piece::Color,
    ) -> Option<&Aspect> {
        let (id, lock) = bit_board.get_key_lock(color);
        let real_key = self.get_real_key(id, lock)?;

        self.inner
            .get(&real_key)
            .map(|(_, aspect_evaluation)| aspect_evaluation)
    }

    fn get_mut_aspect_evaluation(&mut self, id: u64, lock: u64) -> Option<&mut Aspect> {
        let real_key = self.get_real_key(id, lock)?;

        self.inner
            .get_mut(&real_key)
            .map(|(_, aspect_evaluation)| aspect_evaluation)
    }

    fn get_real_key(&self, id: u64, lock: u64) -> Option<u64> {
        let mut real_key = id;
        if !self.inner.contains_key(&id) {
            return None;
        }

        for index in 0..bit_constant::COLLIDEZOBRISTKEY.len() {
            if self.inner.contains_key(&real_key) && lock == self.inner.get(&real_key).unwrap().0 {
                break;
            }

            assert!(
                false,
                "Key:({id:016x})->RealKey:({real_key:016x})'s Lock({lock:016x}) is not find!\n"
            );

            real_key ^= bit_constant::COLLIDEZOBRISTKEY[index];
        }

        // assert!(
        //     self.inner.contains_key(&real_key),
        //     "Key:({id:016x})->RealKey:({real_key:016x})'s Lock({lock:016x}) is not find!\n"
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
    fn save_to(
        conn: &mut SqliteConnection,
        zorbist_datas: Vec<ZorbistData>,
        aspect_datas: Vec<AspectData>,
        evaluation_datas: Vec<EvaluationData>,
    ) -> Result<(usize, usize, usize), Error> {
        // Sqlite3 "PRAGMA foreign_keys = ON"
        EvaluationData::clear(conn);
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

        Self::save_to(conn, zorbist_datas, aspect_datas, evaluation_datas)
    }

    pub fn from_db_manuals(conn: &mut SqliteConnection) -> Result<Self, Error> {
        let mut zorbist = Zorbist::new();
        let bit_board = crate::board::Board::new().bit_board();
        for info in ManualInfo::from_db(conn, "%")? {
            if let Some(rowcols) = info.rowcols {
                for (id, lock, aspect) in bit_board.clone().get_id_lock_asps(rowcols) {
                    zorbist.insert(id, lock, aspect);
                }
            }
        }

        Ok(zorbist)
    }

    pub fn from_history_datas(history_datas: &Vec<models::HistoryData>) -> Self {
        let mut zorbist = Self::new();
        for history_data in history_datas {
            zorbist.insert(
                history_data.akey as u64,
                history_data.lock as u64,
                Aspect::from(
                    history_data.from_index as usize,
                    ToIndex::from(
                        history_data.to_index as usize,
                        Evaluation::from(history_data.count as usize),
                    ),
                ),
            );
        }

        zorbist
    }

    pub fn get_history_datas(&self) -> Vec<models::HistoryData> {
        let mut history_datas = vec![];
        for (akey, (lock, aspect)) in self.inner.iter() {
            for (from_index, to_index_eval) in aspect.inner.iter() {
                for (to_index, eval) in to_index_eval.inner.iter() {
                    history_datas.push(models::HistoryData::from((
                        *akey,
                        *lock,
                        *from_index,
                        *to_index,
                        eval.count,
                    )));
                }
            }
        }

        history_datas
    }

    pub fn to_string(&self) -> String {
        let mut result = String::new();
        for (id, (lock, aspect_evaluation)) in self.inner.iter() {
            result.push_str(&format!("id:  {} lock: {}\n", id, lock));
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
    use crate::manual;
    use crate::models;

    #[test]
    #[ignore = "从文件提取zorbist后存入数据库"]
    fn test_eval_from_file() {
        let filename_manuals = crate::common::get_filename_manuals();
        let manuals = filename_manuals
            .into_iter()
            .map(|(_, _, manual)| manual)
            .collect::<Vec<manual::Manual>>();
        let zorbist = manual::get_zorbist_manuals(manuals);
        let result = zorbist.to_string();
        std::fs::write(format!("tests/output/zobrist_file.txt"), result).expect("Write Err.");

        let conn = &mut models::get_conn();
        let result = zorbist.save_db(conn).expect("Save Err.");
        println!("Save_from_file zor_asp_eva: {:?}", result);
    }

    #[test]
    #[ignore = "从数据库提取manuals后转换成zorbist. (4.25s)"]
    fn test_eval_from_db_manuals() {
        let conn = &mut models::get_conn();
        let zorbist = Zorbist::from_db_manuals(conn).unwrap();
        println!("From_db_manuals: {}", (zorbist.len()));
    }

    #[test]
    #[ignore = "从数据库提取manuals转换成的zorbist_datas再存入数据库. (25.89)"]
    fn test_eval_save_db() {
        let conn = &mut models::get_conn();
        let zorbist = Zorbist::from_db_manuals(conn).unwrap();
        println!("From_db_manuals: {}", (zorbist.len()));

        let result = zorbist.save_db(conn);
        println!("Save_from_db_manuals zor_asp_eva: {:?}", result);

        let man_count = models::ManualInfo::count(conn).unwrap();
        let zor_count = models::ZorbistData::count(conn).unwrap();
        let asp_count = models::AspectData::count(conn).unwrap();
        let eva_count = models::EvaluationData::count(conn).unwrap();
        println!(
            "manual_info count: {} zor_asp_eva: {:?}",
            man_count,
            (zor_count, asp_count, eva_count)
        );
    }

    #[test]
    #[ignore = "从数据库zorbist表直接提取zorbist. (8,63s)"]
    fn test_eval_from_db() {
        let conn = &mut models::get_conn();
        let zorbist = Zorbist::from_db(conn).unwrap();
        println!("zorbist len: {}", zorbist.len());
    }

    #[test]
    #[ignore = "从数据表提取history_datas(2.12s), 转换为zorbist(4.22s), 存入数据库(12.52s). (18.86s)"]
    fn test_eval_from_db_history() {
        let conn = &mut models::get_conn();
        let history_datas = models::HistoryData::from_db(conn).unwrap();
        let zorbist = Zorbist::from_history_datas(&history_datas);
        println!("zorbist len: {}", zorbist.len());

        let history_datas = zorbist.get_history_datas();
        let _ = models::HistoryData::save_db(conn, &history_datas);
        println!("history_datas len: {}", history_datas.len());
    }
}
