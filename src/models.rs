#![allow(dead_code)]

// use diesel;
use crate::schema::manual;
use diesel::prelude::*;

// #[derive(Queryable, Selectable)]
// #[diesel(table_name = crate::schema::manual)]
// #[diesel(check_for_backend(diesel::sqlite::Sqlite))]
// pub struct ManualInfoData {
//     pub id: i32,
//     pub source: String,
//     pub title: String,
//     pub game: String,
//     pub date: String,
//     pub site: String,
//     pub black: String,
//     pub rowcols: String,
//     pub red: String,
//     pub eccosn: String,
//     pub ecconame: String,
//     pub win: String,
//     pub opening: String,
//     pub writer: String,
//     pub author: String,
//     pub atype: String,
//     pub version: String,
//     pub fen: String,
//     pub movestring: String,
// }

#[derive(Insertable, Selectable)]
#[diesel(table_name = manual)]
pub struct NewManual<'a> {
    pub title: &'a str,
    pub game: &'a str,
}

use diesel::sqlite::SqliteConnection;
use dotenvy::dotenv;
use std::env;

pub fn establish_connection() -> SqliteConnection {
    use diesel::prelude::*;
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    SqliteConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}

// use self::models::{NewPost, Post};

pub fn insert_manual(conn: &mut SqliteConnection, title: &str, game: &str) {
    let new_manual = NewManual { title, game };

    diesel::insert_into(manual::table)
        .values(&new_manual)
        // .returning(NewManual::as_returning())
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

        let _ = insert_manual(connection, title, game);
        // println!(
        //     "\nSaved title: {} game: {}",
        //     new_manual.title, new_manual.game
        // );
    }
}
