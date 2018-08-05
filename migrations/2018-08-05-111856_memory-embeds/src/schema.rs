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
    guilds (guild_id) {
        guild_id -> Int8,
        prefix -> Nullable<Text>,
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

joinable!(definitions -> keywords (keyword));

allow_tables_to_appear_in_same_query!(
    definitions,
    guilds,
    keywords,
);
