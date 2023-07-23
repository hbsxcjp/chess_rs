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

    let manual_fields_str = manual_fields()
        .iter()
        .map(|field| field.to_owned() + " TEXT")
        .collect::<Vec<String>>()
        .join(", ");

    let zorbist_fields_str = zorbist_fields()
        .iter()
        .map(|field| field.to_owned() + " INTEGER NOT NULL")
        .collect::<Vec<String>>()
        .join(", ");

    conn.execute(&create_table(MANUAL_TABLE, &manual_fields_str), [])?;
    conn.execute(&create_table(ZORBIST_TABLE, &zorbist_fields_str), [])?;
    Ok(())
}

pub fn clear_table(conn: &mut Connection, table: &str) -> Result<()> {
    for sql in [
        format!("DELETE FROM {table}"),
        format!("UPDATE sqlite_sequence SET seq = 0 WHERE name = '{table}'"),
    ] {
        conn.execute(&sql, [])?;
    }

    Ok(())
}

pub fn insert_manuals(
    conn: &mut Connection,
    filename_manuals: &Vec<(&str, &str, manual::Manual)>,
) -> Result<()> {
    let transcation = conn.transaction()?;

    for (filename, _, manual) in filename_manuals {
        let mut info = manual.get_info();
        info.insert(manual::InfoKey::Source.to_string(), filename.to_string());
        info.insert(manual::InfoKey::RowCols.to_string(), manual.to_rowcols());
        info.insert(
            manual::InfoKey::MoveString.to_string(),
            manual.get_manualmove_string(coord::RecordType::PgnZh),
        );

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

pub fn insert_xqbase_manuals(conn: &mut Connection, cond: &str) -> Result<()> {
    let transcation = conn.transaction()?;
    let manual_fields = manual_fields();
    let manual_fields_str = manual_fields.join(", ");

    let other_conn = Connection::open("tests/output/other_data.db").unwrap();
    let other_sql = format!("SELECT {manual_fields_str} FROM {MANUAL_TABLE} WHERE {cond}");

    let mut stmt = other_conn.prepare(&other_sql).unwrap();
    let value_iter = stmt
        .query_map([], |row| {
            let mut values: Vec<String> = vec![];
            for index in 0..manual_fields.len() {
                let value = if let Ok(val) = row.get::<usize, String>(index) {
                    val
                } else {
                    String::new()
                };
                values.push(format!("'{value}'"));
            }

            Ok(values.join(", "))
        })
        .unwrap();

    for values_res in value_iter {
        if let Ok(values) = values_res {
            let sql = format!("INSERT INTO {MANUAL_TABLE} ({manual_fields_str}) VALUES ({values})");
            transcation.execute(&sql, [])?;
        }
    }

    transcation.commit()
}

pub fn get_manuals(conn: &mut Connection, cond: &str) -> Vec<manual::Manual> {
    let manual_fields = manual_fields();
    let manual_fields_str = manual_fields.join(", ");
    let sql = format!("SELECT {manual_fields_str} FROM {MANUAL_TABLE} WHERE {cond}");
    let mut stmt = conn.prepare(&sql).unwrap();
    let info_iter = stmt
        .query_map([], |row| {
            let mut info = manual::ManualInfo::new();
            for (index, field) in manual_fields.iter().enumerate() {
                if let Ok(value) = row.get::<usize, String>(index) {
                    info.insert(field.to_string(), value);
                }
            }

            // println!("{:?}", info);
            Ok(info)
        })
        .unwrap();

    let mut manuals = Vec::<manual::Manual>::new();
    for info_opt in info_iter {
        if let Ok(manual) = manual::Manual::from_info(info_opt.unwrap()) {
            manuals.push(manual);
        }
    }

    manuals
}

pub fn insert_zorbist_evaluation(
    conn: &mut Connection,
    zorbist_eval: &evaluation::ZorbistEvaluation,
) -> Result<()> {
    let transcation = conn.transaction()?;

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

    let mut data_values = Vec::<(u64, u64, usize, usize, usize)>::new();
    for row_data in row_iter {
        data_values.push(row_data.unwrap());
    }

    evaluation::ZorbistEvaluation::from_data_values(data_values)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::common;

    #[test]
    fn test_database() {
        let filename_manuals = common::get_filename_manuals();

        // 初始化数据表
        let mut conn = get_connection();
        let _ = init_database(&conn).map_err(|err| assert!(false, "init_database: {:?}!\n", err));

        // 文件棋谱
        let _ = clear_table(&mut conn, MANUAL_TABLE);
        let _ = insert_manuals(&mut conn, &filename_manuals)
            .map_err(|err| assert!(false, "insert_manuals: {:?}!\n", err));
        let manuals = get_manuals(&mut conn, "id < 6");
        for (index, manual) in manuals.iter().enumerate() {
            assert!(filename_manuals[index].2 == *manual);
        }

        // 网络棋谱
        // let _ = clear_table(&mut conn, MANUAL_TABLE);
        let _ = insert_xqbase_manuals(&mut conn, "id > 100000 AND id < 100006").unwrap(); //
        let xqbase_manuals = get_manuals(&mut conn, "id > 5 AND id < 11");
        for (index, manual) in xqbase_manuals.iter().enumerate() {
            std::fs::write(
                format!("tests/output/xqbase_{index}.txt"),
                manual.to_string(coord::RecordType::PgnZh),
            )
            .expect("Write Err.");
        }

        // 历史棋谱
        let _ = clear_table(&mut conn, ZORBIST_TABLE);
        // manuals.append(&mut xqbase_manuals);
        let manuals = get_manuals(&mut conn, "id > 5 AND id < 11"); // AND id < 11
        let manuals = manuals
            .iter()
            .map(|manual| manual)
            .collect::<Vec<&manual::Manual>>();
        let zorbist_evaluation = common::get_zorbist_evaluation(manuals);
        let _ = insert_zorbist_evaluation(&mut conn, &zorbist_evaluation)
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
