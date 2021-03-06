table! {
    attributes (name, pin) {
        pin -> Int8,
        name -> Text,
        value -> Int4,
        maximum -> Int4,
    }
}

table! {
    bot (id) {
        id -> Int8,
        activity_type -> crate::db::model::ActivityKindMapping,
        activity_name -> Text,
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
    notes (name, pin) {
        pin -> Int8,
        name -> Text,
        note -> Text,
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
joinable!(notes -> characters (pin));

allow_tables_to_appear_in_same_query!(
    attributes,
    bot,
    channels,
    characters,
    definitions,
    keywords,
    notes,
    prefixes,
);
