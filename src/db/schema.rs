table! {
    attributes (name, pin) {
        pin -> Int8,
        name -> Text,
        value -> Int4,
        maximum -> Int4,
    }
}

table! {
    channels (channel) {
        channel -> Int8,
        gm -> Int8,
    }
}

table! {
    characters (pin) {
        name -> Text,
        channel -> Int8,
        owner -> Int8,
        pin -> Int8,
    }
}

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

joinable!(attributes -> characters (pin));
joinable!(definitions -> keywords (keyword));

allow_tables_to_appear_in_same_query!(
    attributes,
    channels,
    characters,
    definitions,
    keywords,
    prefixes,
);
