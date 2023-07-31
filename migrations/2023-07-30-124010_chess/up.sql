-- Your SQL goes here

CREATE TABLE manual (id INTEGER PRIMARY KEY AUTOINCREMENT, source TEXT, title TEXT, game TEXT, date TEXT, site TEXT, black TEXT, rowcols TEXT, red TEXT, eccosn TEXT, ecconame TEXT, win TEXT, opening TEXT, writer TEXT, author TEXT, atype TEXT, version TEXT, fen TEXT, movestring TEXT);

CREATE TABLE zorbist ( id INTEGER PRIMARY KEY AUTOINCREMENT, key INTEGER, lock INTEGER, from_index INTEGER, to_index INTEGER, count INTEGER );
