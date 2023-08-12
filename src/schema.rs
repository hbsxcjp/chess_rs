// @generated automatically by Diesel CLI.

diesel::table! {
    manual (id) {
        id -> Integer,
        source -> Nullable<Text>,
        title -> Text,
        game -> Text,
        date -> Nullable<Text>,
        site -> Nullable<Text>,
        black -> Nullable<Text>,
        rowcols -> Nullable<Text>,
        red -> Nullable<Text>,
        eccosn -> Nullable<Text>,
        ecconame -> Nullable<Text>,
        win -> Nullable<Text>,
        opening -> Nullable<Text>,
        writer -> Nullable<Text>,
        author -> Nullable<Text>,
        atype -> Nullable<Text>,
        version -> Nullable<Text>,
        fen -> Nullable<Text>,
        movestring -> Nullable<Text>,
    }
}
