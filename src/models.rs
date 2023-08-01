#![allow(dead_code)]

// use diesel;
use crate::schema::aspect;
use crate::schema::evaluation;
use crate::schema::manual;
use crate::schema::zorbist;
use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = aspect)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Aspect {
    pub id: i32,
    pub from_index: i32,
    pub key: i64,
}

#[derive(Insertable, Queryable, Selectable)]
#[diesel(table_name = evaluation)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Evaluation {
    pub to_index: i32,
    pub count: i32,
    pub from_index_id: i32,
}

#[derive(Debug, Insertable, Queryable, Selectable)]
#[diesel(table_name = crate::schema::manual)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct ManualInfo {
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
pub struct Zorbist {
    pub id: i64,
}

impl ManualInfo {
    pub fn new() -> Self {
        ManualInfo {
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
            fen: None,
            movestring: None,
        }
    }
}

pub fn establish_connection() -> SqliteConnection {
    use diesel::prelude::*;
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

pub fn insert_manual(conn: &mut SqliteConnection, title: &str, game: &str) {
    //-> ManualInfoData
    let new_manual = ManualInfo {
        source: None,
        title: title.to_string(),
        game: game.to_string(),
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
        fen: None,
        movestring: None,
    };

    diesel::insert_into(manual::table)
        .values(&new_manual)
        // .returning(ManualInfoData::as_returning())
        // .get_result(conn)
        .execute(conn)
        .expect("Error saving new manual");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // #[ignore = "忽略：初始化数据库表。"]
    fn test() {
        let connection = &mut establish_connection();

        let title = "My title";
        let game = "My game.";

        let manual = insert_manual(connection, title, game);
        println!("Saved : {:?}", manual);
    }
}
