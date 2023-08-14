#![allow(dead_code)]

use crate::board;
// use diesel;
use crate::schema::{self, manual}; //, history  aspect, evaluation,, zorbist
use diesel::connection::SimpleConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::Error;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

const DB_THREADS: usize = 3;

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
pub type SqlitePooledConnection = PooledConnection<ConnectionManager<SqliteConnection>>;

// pub struct DB {
//     conn: Option<SqlitePooledConnection>,
// }

// #[derive(Insertable, Queryable, Selectable)]
// #[diesel(table_name = history)]
// #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
// pub struct HistoryData {
//     pub akey: i64,
//     pub lock: i64,
//     pub from_index: i32,
//     pub to_index: i32,
//     pub count: i32,
// }

// #[derive(Insertable, Queryable, Selectable)]
// #[diesel(table_name = zorbist)]
// #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
// pub struct ZorbistData {
//     pub id: i64,
//     pub lock: i64,
// }

// #[derive(Insertable, Queryable, Selectable, Associations)]
// #[diesel(belongs_to(ZorbistData, foreign_key = zorbist_id))]
// #[diesel(table_name = aspect)]
// #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
// pub struct AspectData {
//     pub id: i32,
//     pub from_index: i32,
//     pub zorbist_id: i64,
// }

// #[derive(Insertable, Queryable, Selectable, Associations)]
// #[diesel(belongs_to(AspectData, foreign_key = aspect_id))]
// #[diesel(table_name = evaluation)]
// #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
// pub struct EvaluationData {
//     // pub id: i32,
//     pub to_index: i32,
//     pub count: i32,
//     pub aspect_id: i32,
// }

#[derive(Insertable, Queryable, Selectable, Debug)]
#[diesel(table_name = manual)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ManualInfo {
    // pub id: i32,
    pub source: Option<String>,
    pub title: String,
    pub game: String,
    pub date: Option<String>,
    pub site: Option<String>,
    pub black: Option<String>,
    pub rowcols: Option<String>,
    pub red: Option<String>,
    pub eccosn: Option<String>,
    pub ecconame: Option<String>,
    pub win: Option<String>,
    pub opening: Option<String>,
    pub writer: Option<String>,
    pub author: Option<String>,
    pub atype: Option<String>,
    pub version: Option<String>,
    pub fen: Option<String>,
    pub movestring: Option<String>,
}

lazy_static! {
    pub static ref SQLITEPOOL: SqlitePool = {
        use diesel::prelude::*;
        dotenv().ok();

        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        // SqliteConnection::establish(&database_url)
        // .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))

        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        Pool::builder().build(manager).unwrap()
    };
}

pub fn get_conn() -> SqlitePooledConnection {
    let mut conn = SQLITEPOOL.get().unwrap();
    conn.batch_execute("PRAGMA foreign_keys = ON; PRAGMA synchronous = OFF;")
        .expect("Set foreign_keys faild.");

    conn
}

fn set_seq_zero(conn: &mut SqliteConnection, table: &str) {
    let _ = conn.batch_execute(&format!(
        "UPDATE sqlite_sequence SET seq = 0 WHERE name = '{table}'"
    ));
}

// impl HistoryData {
//     pub fn clear(conn: &mut SqliteConnection) {
//         let _ = diesel::delete(history::table).execute(conn);
//         set_seq_zero(conn, "history");
//     }

//     pub fn count(conn: &mut SqliteConnection) -> Result<i64, Error> {
//         use diesel::dsl::count;
//         use schema::history::dsl::*;
//         history.select(count(id)).first::<i64>(conn)
//     }

//     pub fn from_db(conn: &mut SqliteConnection) -> Result<Vec<Self>, Error> {
//         history::table.select(Self::as_select()).load::<Self>(conn)
//     }

//     pub fn save_db(conn: &mut SqliteConnection, history_datas: &Vec<Self>) -> Result<usize, Error> {
//         HistoryData::clear(conn);
//         diesel::insert_into(history::table)
//             .values(history_datas)
//             .execute(conn)
//     }

//     pub fn init_xqbase(conn: &mut SqliteConnection) -> Result<usize, Error> {
//         let mut history_datas = vec![];
//         let bit_board = crate::board::Board::new().bit_board();
//         for info in ManualInfo::from_db(conn, "%")? {
//             if let Some(rowcols) = info.rowcols {
//                 for key_value in bit_board.clone().get_key_values(rowcols) {
//                     history_datas.push(HistoryData::from(key_value));
//                 }
//             }
//         }

//         Self::save_db(conn, &history_datas)
//     }

//     pub fn from(key_value: (u64, u64, usize, usize, usize)) -> Self {
//         let (akey, lock, from_index, to_index, count) = key_value;
//         let akey = akey as i64;
//         let lock = lock as i64;
//         let from_index = from_index as i32;
//         let to_index = to_index as i32;
//         let count = count as i32;
//         Self {
//             akey,
//             lock,
//             from_index,
//             to_index,
//             count,
//         }
//     }

//     pub fn get_key_value(&self) -> (i64, i64, i32, i32, i32) {
//         (
//             self.akey,
//             self.lock,
//             self.from_index,
//             self.to_index,
//             self.count,
//         )
//     }
// }

// impl ZorbistData {
//     pub fn count(conn: &mut SqliteConnection) -> Result<i64, Error> {
//         use diesel::dsl::count;
//         use schema::zorbist::dsl::*;
//         zorbist.select(count(id)).first::<i64>(conn)
//     }
// }

// impl AspectData {
//     pub fn max_id(conn: &mut SqliteConnection) -> Result<i32, Error> {
//         use diesel::dsl::max;
//         use schema::aspect::dsl::*;
//         let max_id = aspect.select(max(id)).limit(1).load::<Option<i32>>(conn)?;
//         Ok(max_id[0].unwrap())
//     }

//     pub fn count(conn: &mut SqliteConnection) -> Result<i64, Error> {
//         use diesel::dsl::count;
//         use schema::aspect::dsl::*;
//         aspect.select(count(id)).first::<i64>(conn)
//     }
// }

// impl EvaluationData {
//     pub fn clear(conn: &mut SqliteConnection) {
//         let _ = diesel::delete(zorbist::table).execute(conn);
//         set_seq_zero(conn, "evaluation");
//     }

//     pub fn count(conn: &mut SqliteConnection) -> Result<i64, Error> {
//         use diesel::dsl::count;
//         use schema::evaluation::dsl::*;
//         evaluation.select(count(id)).first::<i64>(conn)
//     }
// }

impl ManualInfo {
    pub fn new() -> Self {
        ManualInfo {
            // id: 0,
            source: None,
            title: String::from("未命名"),
            game: String::from("人机对战"),
            date: None,
            site: None,
            black: None,
            rowcols: None,
            red: None,
            eccosn: None,
            ecconame: None,
            win: None,
            opening: None,
            writer: None,
            author: None,
            atype: None,
            version: None,
            fen: Some(board::FEN.to_string() + " r - - 0 1"),
            movestring: None,
        }
    }

    pub fn from(key_values: Vec<(String, String)>) -> Self {
        let mut info = Self::new();
        for (key, value) in key_values {
            match key {
                _ if key == "title" => info.title = value,
                _ if key == "game" => info.game = value,
                _ if key == "source" => info.source = Some(value),
                _ if key == "date" => info.date = Some(value),
                _ if key == "site" => info.site = Some(value),
                _ if key == "black" => info.black = Some(value),
                _ if key == "rowcols" => info.rowcols = Some(value),
                _ if key == "red" => info.red = Some(value),
                _ if key == "eccosn" => info.eccosn = Some(value),
                _ if key == "ecconame" => info.ecconame = Some(value),
                _ if key == "win" => info.win = Some(value),
                _ if key == "opening" => info.opening = Some(value),
                _ if key == "writer" => info.writer = Some(value),
                _ if key == "author" => info.author = Some(value),
                _ if key == "atype" => info.atype = Some(value),
                _ if key == "version" => info.version = Some(value),
                _ if key == "fen" => info.fen = Some(value),
                _ if key == "movestring" => info.movestring = Some(value),
                _ => (),
            }
        }

        info
    }

    pub fn get_key_values(&self) -> Vec<(&'static str, &String)> {
        let mut result = vec![("title", &self.title), ("game", &self.game)];
        for (key, value) in [
            ("source", &self.source),
            ("date", &self.date),
            ("site", &self.site),
            ("black", &self.black),
            ("rowcols", &self.rowcols),
            ("red", &self.red),
            ("eccosn", &self.eccosn),
            ("ecconame", &self.ecconame),
            ("win", &self.win),
            ("opening", &self.opening),
            ("writer", &self.writer),
            ("author", &self.author),
            ("atype", &self.atype),
            ("version", &self.version),
            ("fen", &self.fen),
            ("movestring", &self.movestring),
        ] {
            if let Some(value) = value.as_ref() {
                result.push((key, value));
            }
        }

        result
    }

    pub fn clear(conn: &mut SqliteConnection) {
        let _ = diesel::delete(manual::table).execute(conn);
        set_seq_zero(conn, "manual");
    }

    pub fn count(conn: &mut SqliteConnection) -> Result<i64, Error> {
        use diesel::dsl::count;
        use schema::manual::dsl::*;
        manual.select(count(id)).first::<i64>(conn)
    }

    pub fn init_xqbase(conn: &mut SqliteConnection) -> Result<i64, Error> {
        ManualInfo::clear(conn);
        let query = std::fs::read_to_string("insert_xqbase.sql").unwrap();
        let _ = conn.batch_execute(&query);

        ManualInfo::count(conn)
    }

    pub fn from_db(conn: &mut SqliteConnection, title_part: &str) -> Result<Vec<Self>, Error> {
        manual::table
            .filter(manual::title.like(title_part))
            .select(Self::as_select())
            .load::<Self>(conn)
    }

    pub fn save_db(infos: &Vec<ManualInfo>, conn: &mut SqliteConnection) -> Result<usize, Error> {
        diesel::insert_into(manual::table)
            .values(infos)
            .execute(conn)
    }

    pub fn get_fen(&self) -> &str {
        if let Some(value) = &self.fen {
            if let Some((fen, _)) = value.split_once(" ") {
                return fen;
            }
        }

        board::FEN
    }

    pub fn get_rowcols(conn: &mut SqliteConnection) -> Result<Vec<Option<String>>, Error> {
        // use diesel::dsl::max;
        use schema::manual::dsl::*;
        manual.select(rowcols.as_sql()).load::<Option<String>>(conn)
    }

    pub fn get_copy(&self) -> Self {
        Self::from(
            self.get_key_values()
                .iter()
                .map(|(key, value)| (key.to_string(), value.to_string()))
                .collect(),
        )
    }

    pub fn set_source_moves(&mut self, source: &str, rowcols: &str, movestring: &str) {
        self.source = Some(source.to_string());
        self.rowcols = Some(rowcols.to_string());
        self.movestring = Some(movestring.to_string());
    }

    pub fn cut_source_moves(&mut self) {
        self.source = None;
        self.rowcols = None;
        self.movestring = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore = "测试manualinfo模型"]
    fn test_manualinfo() {
        let conn = &mut get_conn();

        let infos = vec![ManualInfo::new()];
        let count = ManualInfo::save_db(&infos, conn).unwrap_or(0);
        println!("Saved : {:?}", count);
    }

    #[test]
    #[ignore = "从insert_xqbase.sql文件提取SQL语句运行将12141个manual存入数据库。(神速！)"]
    fn test_init_xqbase_manuals() {
        let conn = &mut get_conn();
        let result = ManualInfo::init_xqbase(conn);
        println!("ManualInfo::init_xqbase count: {}", result.unwrap());
    }

    #[test]
    #[ignore = "从数据库提取全部manuals的rowcols存入文本文件。"]
    fn test_init_xqbase_rowcols() {
        let conn = &mut get_conn();
        let mut result = String::new();
        for rowcols in ManualInfo::get_rowcols(conn).unwrap() {
            if let Some(rowcols) = rowcols {
                result.push_str(&format!("\t\"{}\",\n", rowcols));
            }
        }
        std::fs::write(
            format!("tests/output/manuals_rowcols.txt"),
            // format!("src/manual_rowcols.rs"),
            format!(
                "#![allow(dead_code)]\n\npub const ROWCOLS: [&str; 12141] = [\n{}];",
                result
            ),
        )
        .expect("Write Err.");
    }
}
