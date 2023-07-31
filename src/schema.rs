// @generated automatically by Diesel CLI.

diesel::table! {
    manual (id) {
        id -> Nullable<Integer>,
        source -> Nullable<Text>,
        title -> Nullable<Text>,
        game -> Nullable<Text>,
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

diesel::table! {
    zorbist (id) {
        id -> Nullable<Integer>,
        key -> Nullable<Integer>,
        lock -> Nullable<Integer>,
        from_index -> Nullable<Integer>,
        to_index -> Nullable<Integer>,
        count -> Nullable<Integer>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    manual,
    zorbist,
);
