#![allow(dead_code)]

use crate::board;
// use diesel;
use crate::schema::aspect;
use crate::schema::evaluation;
use crate::schema::manual;
use crate::schema::zorbist;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

pub const MANUAL_FIELD_NUM: u32 = 18;

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = aspect)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AspectData {
    pub id: i32,
    pub from_index: i32,
    pub key: i64,
}

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = evaluation)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct EvaluationData {
    pub to_index: i32,
    pub count: i32,
    pub from_index_id: i32,
}

#[derive(Debug, Insertable, Queryable, Selectable)]
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

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = zorbist)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ZorbistData {
    pub id: i64,
}

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
            fen: Some(board::FEN.to_string()),
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

    pub fn save_to(&self, conn: &mut SqliteConnection) -> bool {
        diesel::insert_into(manual::table)
            .values(self)
            .execute(conn)
            .is_ok()
    }
}

pub fn establish_connection() -> SqliteConnection {
    use diesel::prelude::*;
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // #[ignore = "忽略：插入数据"]
    fn test() {
        let conn = &mut establish_connection();

        let result = ManualInfo::new().save_to(conn);
        println!("Saved : {:?}", result);
    }
}
