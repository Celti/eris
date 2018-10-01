table! {
    definitions (keyword, definition) {
        keyword -> Text,
        definition -> Text,
        submitter -> Int8,
        timestamp -> Timestamptz,
        embedded -> Bool,
    }
}

table! {
    keywords (keyword) {
        keyword -> Text,
        owner -> Int8,
        bareword -> Bool,
        hidden -> Bool,
        protect -> Bool,
        shuffle -> Bool,
    }
}

table! {
    prefixes (id) {
        id -> Int8,
        prefix -> Text,
    }
}

joinable!(definitions -> keywords (keyword));

allow_tables_to_appear_in_same_query!(
    definitions,
    keywords,
    prefixes,
);
