-- Your SQL goes here

CREATE TABLE aspect (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, 
    from_index INTEGER NOT NULL, 

    zorbist_id BIGINT NOT NULL, 
    FOREIGN KEY (zorbist_id) REFERENCES zorbist(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE evaluation (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, 
    to_index INTEGER NOT NULL, 
    count INTEGER NOT NULL,

    aspect_id INTEGER NOT NULL, 
    FOREIGN KEY (aspect_id) REFERENCES aspect(id) ON DELETE CASCADE ON UPDATE CASCADE
);

CREATE TABLE manual (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, 
    source TEXT, 
    title TEXT NOT NULL, 
    game TEXT NOT NULL, 
    date TEXT, 
    site TEXT, 
    black TEXT, 
    rowcols TEXT, 
    red TEXT, 
    eccosn TEXT, 
    ecconame TEXT, 
    win TEXT, 
    opening TEXT, 
    writer TEXT, 
    author TEXT, 
    atype TEXT, 
    version TEXT, 
    fen TEXT, 
    movestring TEXT
);

CREATE TABLE zorbist (
    id BIGINT PRIMARY KEY NOT NULL
);
