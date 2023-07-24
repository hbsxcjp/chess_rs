#![allow(dead_code)]

use crate::{coord, evaluation, manual};
use rusqlite::{params, Connection, Result};

const MANUAL_TABLE: &str = "manual";
const ZORBIST_TABLE: &str = "zorbist";
const BASE_FILE: &str = "tests/output/data.db";

fn manual_fields() -> Vec<String> {
    (0..=(manual::InfoKey::MoveString as usize))
        .map(|index| manual::InfoKey::try_from(index).unwrap().to_string())
        .collect::<Vec<String>>()
}

fn zorbist_fields() -> Vec<String> {
    ["key", "lock", "from_index", "to_index", "count"]
        .iter()
        .map(|&field| field.to_owned())
        .collect::<Vec<String>>()
}

pub fn get_connection() -> Connection {
    Connection::open(BASE_FILE).unwrap()
}

pub fn init_database(conn: &Connection) -> Result<()> {
    let create_table = |table: &str, fields: &str| -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {table} (id INTEGER PRIMARY KEY AUTOINCREMENT, {fields})"
        )
    };

    let manual_fields = manual_fields()
        .iter()
        .map(|field| field.to_owned() + " TEXT")
        .collect::<Vec<String>>()
        .join(", ");
    let zorbist_fields = zorbist_fields()
        .iter()
        .map(|field| field.to_owned() + " INTEGER NOT NULL")
        .collect::<Vec<String>>()
        .join(", ");

    conn.execute(&create_table(MANUAL_TABLE, &manual_fields), [])?;
    conn.execute(&create_table(ZORBIST_TABLE, &zorbist_fields), [])?;
    Ok(())
}

fn insert_manual_infos(conn: &mut Connection, infos: Vec<manual::ManualInfo>) -> Result<()> {
    let transcation = conn.transaction()?;
    for info in infos {
        let keys = info.keys().cloned().collect::<Vec<String>>().join(", ");
        let values = info
            .values()
            .map(|value| format!("'{value}'"))
            .collect::<Vec<String>>()
            .join(", ");
        let sql = format!("INSERT INTO {MANUAL_TABLE} ({keys}) VALUES ({values})");

        transcation.execute(&sql, [])?;
    }

    transcation.commit()
}

pub fn insert_manuals(
    conn: &mut Connection,
    filename_manuals: Vec<(&str, &str, manual::Manual)>,
) -> Result<()> {
    let mut infos = Vec::<manual::ManualInfo>::new();
    for (filename, _, manual) in filename_manuals {
        let mut info = manual.get_info();
        info.insert(manual::InfoKey::Source.to_string(), filename.to_string());
        info.insert(manual::InfoKey::RowCols.to_string(), manual.to_rowcols());
        info.insert(
            manual::InfoKey::MoveString.to_string(),
            manual.to_string(coord::RecordType::Txt),
        );
        infos.push(info);
    }

    insert_manual_infos(conn, infos)
}

pub fn init_zorbist_evaluation(
    conn: &mut Connection,
    zorbist_eval: &evaluation::ZorbistEvaluation,
) -> Result<()> {
    let transcation = conn.transaction()?;
    for sql in [
        format!("DELETE FROM {ZORBIST_TABLE}"),
        format!("UPDATE sqlite_sequence SET seq = 0 WHERE name = '{ZORBIST_TABLE}'"),
    ] {
        transcation.execute(&sql, [])?;
    }

    let zorbist_fields = zorbist_fields().join(", ");
    let sql = format!("INSERT INTO {ZORBIST_TABLE} ({zorbist_fields}) VALUES (?1, ?2, ?3, ?4, ?5)");
    for (key, lock, from_index, to_index, count) in zorbist_eval.get_data_values() {
        transcation.execute(
            &sql,
            params![
                key as i64,
                lock as i64,
                from_index as i64,
                to_index as i64,
                count as i64
            ],
        )?;
    }

    transcation.commit()
}

pub fn get_zorbist_evaluation(conn: &mut Connection) -> evaluation::ZorbistEvaluation {
    let zorbist_fields = zorbist_fields().join(", ");
    let sql = format!("SELECT {zorbist_fields} FROM {ZORBIST_TABLE}");
    let mut stmt = conn.prepare(&sql).unwrap();
    let row_iter = stmt
        .query_map([], |row| {
            Ok((
                row.get::<usize, i64>(0).unwrap() as u64,
                row.get::<usize, i64>(1).unwrap() as u64,
                row.get::<usize, i64>(2).unwrap() as usize,
                row.get::<usize, i64>(3).unwrap() as usize,
                row.get::<usize, i64>(4).unwrap() as usize,
            ))
        })
        .unwrap();

    let mut zorbist_evaluation = evaluation::ZorbistEvaluation::new();
    for row in row_iter {
        if let Ok(values) = row {
            zorbist_evaluation.insert_values(values);
        }
    }

    zorbist_evaluation
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common;

    #[test]
    fn test_database() {
        let filename_manuals = common::get_filename_manuals();
        let zorbist_evaluation = common::get_some_zorbist_evaluation(&filename_manuals);

        let mut conn = get_connection();
        let _ = init_database(&conn).map_err(|err| assert!(false, "init_database: {:?}!\n", err));
        let _ = insert_manuals(&mut conn, filename_manuals)
            .map_err(|err| assert!(false, "insert_manuals: {:?}!\n", err));
        let _ = init_zorbist_evaluation(&mut conn, &zorbist_evaluation)
            .map_err(|err| assert!(false, "insert_zorbist_evaluation: {:?}!\n", err));

        let zorbist_eval_from_database = get_zorbist_evaluation(&mut conn);
        let result = zorbist_eval_from_database.to_string();
        std::fs::write(
            format!("tests/output/zorbist_eval_from_database.txt"),
            result,
        )
        .expect("Write Err.");
    }
}
