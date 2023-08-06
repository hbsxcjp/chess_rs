#![allow(dead_code)]

extern crate rayon;
// use rayon::prelude::*;
// use std::collections::HashMap;

// use crate::evaluation;
// use rusqlite::{params, Connection, Result};

// const MANUAL_TABLE: &str = "manual";
// const ZORBIST_TABLE: &str = "zorbist";
// const BASE_FILE: &str = "tests/output/data.db";

// fn zorbist_fields() -> Vec<String> {
//     ["key", "lock", "from_index", "to_index", "count"]
//         .iter()
//         .map(|&field| field.to_owned())
//         .collect::<Vec<String>>()
// }

// pub fn get_connection() -> Connection {
//     Connection::open(BASE_FILE).unwrap()
// }

// pub fn clear_table(conn: &mut Connection, table: &str) -> Result<()> {
//     for sql in [
//         format!("DELETE FROM {table}"),
//         format!("UPDATE sqlite_sequence SET seq = 0 WHERE name = '{table}'"),
//     ] {
//         conn.execute(&sql, [])?;
//     }

//     Ok(())
// }

// pub fn insert_zorbist(conn: &mut Connection, zorbist_eval: &evaluation::Zorbist) -> Result<()> {
//     let transcation = conn.transaction()?;

//     let zorbist_fields = zorbist_fields().join(", ");
//     let sql = format!("INSERT INTO {ZORBIST_TABLE} ({zorbist_fields}) VALUES (?1, ?2, ?3, ?4, ?5)");
//     for (key, lock, from_index, to_index, count) in zorbist_eval.get_data_values() {
//         transcation.execute(
//             &sql,
//             params![
//                 key as i64,
//                 lock as i64,
//                 from_index as i64,
//                 to_index as i64,
//                 count as i64
//             ],
//         )?;
//     }

//     transcation.commit()
// }

// pub fn get_zorbist(conn: &mut Connection) -> evaluation::Zorbist {
//     let zorbist_fields = zorbist_fields().join(", ");
//     let sql = format!("SELECT {zorbist_fields} FROM {ZORBIST_TABLE}");
//     let mut stmt = conn.prepare(&sql).unwrap();
//     let row_iter = stmt
//         .query_map([], |row| {
//             Ok((
//                 row.get::<usize, i64>(0).unwrap() as u64,
//                 row.get::<usize, i64>(1).unwrap() as u64,
//                 row.get::<usize, i64>(2).unwrap() as usize,
//                 row.get::<usize, i64>(3).unwrap() as usize,
//                 row.get::<usize, i64>(4).unwrap() as usize,
//             ))
//         })
//         .unwrap();

//     let mut zorbist = evaluation::Zorbist::new();
//     for row in row_iter {
//         if let Ok((key, lock, from_index, to_index, count)) = row {
//             zorbist.insert(
//                 key,
//                 lock,
//                 evaluation::Aspect::from(
//                     from_index,
//                     evaluation::ToIndex::from(to_index, evaluation::Evaluation::from(count)),
//                 ),
//             );
//         }
//     }

//     zorbist
// }

// pub fn get_zorbist_rowcols(conn: &mut Connection, cond: &str) -> evaluation::Zorbist {
//     let mut zorbist = evaluation::Zorbist::new();

//     let sql = format!("SELECT RowCols FROM {MANUAL_TABLE} WHERE {cond}");
//     let mut stmt = conn.prepare(&sql).unwrap();
//     let rowcols_iter = stmt
//         .query_map([], |row| row.get::<usize, String>(0))
//         .unwrap();

//     let bit_board = crate::board::Board::new().bit_board();
//     for rowcols in rowcols_iter {
//         if let Ok(rowcols) = rowcols {
//             zorbist.append(bit_board.clone().get_zorbist_rowcols(rowcols));
//         }
//     }

//     zorbist
// }

// #[cfg(test)]
// mod tests {
//     // use super::*;

//     #[test]
//     #[ignore = "忽略：将zorbist存入数据库表。"]
//     // 历史棋谱存入数据库
//     fn test_database_insert_zorbist() {
//         // let mut conn = get_connection();
//         // let mut zorbist = evaluation::ZorbistEvaluation::new();

//         // let _ = clear_table(&mut conn, ZORBIST_TABLE);
//         // let test_from_manuals = false;
//         // zorbist.append(if test_from_manuals {
//         //     let manuals = get_manuals(&mut conn, "id < 11"); //id > 5 AND
//         //     manual::get_zorbist_manuals(manuals)
//         // } else {
//         //     // println!("rowcols_vec: {}", rowcols_vec.len());
//         //     get_zorbist_rowcols(&mut conn, "id > 5 AND id < 110000")
//         //     // 12146
//         // });

//         // let _ = insert_zorbist(&mut conn, &zorbist)
//         //     .map_err(|err| assert!(false, "insert_zorbist: {:?}!\n", err));
//     }

//     #[test]
//     #[ignore = "忽略：从数据库提取zorbist。"]
//     // 从数据库提取
//     fn test_database_get_zorbist() {
//         // let mut conn = get_connection();
//         // let zorbist = get_zorbist(&mut conn);

//         // println!("zorbist: {}", zorbist.len());
//     }
// }
