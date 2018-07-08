table! {
    characters (name, game) {
        char_id -> Int4,
        pinned -> Nullable<Int8>,
        mtime -> Timestamptz,
        name -> Text,
        game -> Text,
    }
}

table! {
    char_base (char_id) {
        char_id -> Int4,
        cur_hp -> Int4,
        max_hp -> Int4,
        cur_rp -> Int4,
        max_rp -> Int4,
        cur_fp -> Int4,
        max_fp -> Int4,
        cur_lfp -> Int4,
        max_lfp -> Int4,
        cur_sp -> Int4,
        max_sp -> Int4,
        cur_lsp -> Int4,
        max_lsp -> Int4,
        cur_ip -> Int4,
        max_ip -> Int4,
        xp -> Int4,
    }
}

table! {
    char_notes (char_id, key) {
        char_id -> Int4,
        key -> Text,
        value -> Text,
    }
}

table! {
    char_points (char_id, key) {
        char_id -> Int4,
        maximum -> Int4,
        value -> Int4,
        key -> Text,
    }
}

table! {
    guilds (guild_id) {
        guild_id -> Int8,
        prefix -> Nullable<Text>,
    }
}

allow_tables_to_appear_in_same_query!(
    characters,
    char_base,
    char_notes,
    char_points,
    guilds,
);
