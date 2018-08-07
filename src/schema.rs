table! {
    definitions (keyword, definition) {
        keyword -> Text,
        definition -> Text,
        submitter -> Int8,
        timestamp -> Timestamptz,
        embed -> Bool,
    }
}

table! {
    keywords (keyword) {
        keyword -> Text,
        owner -> Nullable<Int8>,
        shuffle -> Bool,
        protect -> Bool,
        hidden -> Bool,
        bare -> Bool,
    }
}

table! {
    prefixes (id) {
        id -> Int8,
        prefix -> Nullable<Text>,
    }
}

joinable!(definitions -> keywords (keyword));

allow_tables_to_appear_in_same_query!(
    definitions,
    keywords,
    prefixes,
);
