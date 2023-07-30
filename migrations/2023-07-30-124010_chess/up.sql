-- Your SQL goes here
CREATE TABLE manual (
    id INTEGER PRIMARY KEY AUTOINCREMENT, 
    Source TEXT, 
    Title TEXT, 
    Game TEXT, 
    Date TEXT, 
    Site TEXT, 
    Black TEXT, 
    RowCols TEXT, 
    Red TEXT, 
    EccoSn TEXT, 
    EccoName TEXT, 
    Win TEXT, 
    Opening TEXT, 
    Writer TEXT, 
    Author TEXT, 
    Atype TEXT, 
    Version TEXT, 
    FEN TEXT, 
    MoveString TEXT
);

CREATE TABLE zorbist (
    id INTEGER PRIMARY KEY AUTOINCREMENT, 
    key INTEGER, 
    lock INTEGER, 
    from_index INTEGER, 
    to_index INTEGER, 
    count INTEGER
);
