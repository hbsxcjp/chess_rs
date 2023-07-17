#![allow(dead_code)]

use crate::{evaluation, manual};
use rusqlite::{params, Connection, Result};

const MANUAL_TABLE: &str = "manual";
const ZORBIST_TABLE: &str = "zorbist";
const BASE_FILE: &str = "tests/output/data.db";

pub fn get_connection() -> Connection {
    Connection::open(BASE_FILE).unwrap()
}

pub fn init_database(conn: &Connection) -> Result<()> {
    let create_table = |table: &str, fields: &str| -> String {
        format!(
            "CREATE TABLE IF NOT EXISTS {} (id INTEGER PRIMARY KEY AUTOINCREMENT, {})",
            table, fields
        )
    };

    let manual_fields = (0..=(manual::InfoKey::MoveString as usize))
        .map(|index| format!("{:?} TEXT", manual::InfoKey::try_from(index).unwrap()))
        .collect::<Vec<String>>()
        .join(", ");
    let zorbist_fields =
        "key INTEGER, lock INTEGER, from_index INTEGER, to_index INTEGER, count INTEGER";

    conn.execute(&create_table(MANUAL_TABLE, &manual_fields), [])?;
    conn.execute(&create_table(ZORBIST_TABLE, zorbist_fields), [])?;
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

        transcation.execute(
            "INSERT INTO manual (:keys) VALUES (:values)",
            &[(":keys", &keys), (":values", &values)],
        )?;
    }

    transcation.commit()
}

pub fn insert_manuals(
    conn: &mut Connection,
    filename_manuals: Vec<(&str, manual::Manual)>,
) -> Result<()> {
    let mut infos = Vec::<manual::ManualInfo>::new();
    for (filename, mut manual) in filename_manuals {
        manual.info.insert(
            format!("{:?}", manual::InfoKey::Source),
            filename.to_string(),
        );
        infos.push(manual.info);
    }

    insert_manual_infos(conn, infos)
}

pub fn init_zorbist_evaluation(
    conn: &mut Connection,
    zorbist_eval: &evaluation::ZorbistEvaluation,
) -> Result<()> {
    let transcation = conn.transaction()?;
    transcation.execute("DELETE FROM zorbist", [])?;
    transcation.execute(
        "UPDATE sqlite_sequence SET seq = 0 WHERE name = 'zorbist'",
        [],
    )?;

    for (key, lock, from_index, to_index, count) in zorbist_eval.get_data_values() {
        transcation.execute(
            "INSERT INTO zorbist (key, lock, from_index, to_index, count) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![key as i64, lock as i64, from_index as i64, to_index as i64, count as i64],
        )?;
    }

    transcation.commit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database() {
        let conn = get_connection();
        let _ = init_database(&conn).map_err(|err| assert!(false, "init_database: {:?}!\n", err));
    }
}
