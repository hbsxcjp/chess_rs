// @generated automatically by Diesel CLI.

diesel::table! {
    aspect (id) {
        id -> Integer,
        from_index -> Integer,
        zorbist_id -> BigInt,
    }
}

diesel::table! {
    evaluation (id) {
        id -> Integer,
        to_index -> Integer,
        count -> Integer,
        aspect_id -> Integer,
    }
}

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

diesel::table! {
    zorbist (id) {
        id -> BigInt,
    }
}

diesel::joinable!(aspect -> zorbist (zorbist_id));
diesel::joinable!(evaluation -> aspect (aspect_id));

diesel::allow_tables_to_appear_in_same_query!(
    aspect,
    evaluation,
    manual,
    zorbist,
);
