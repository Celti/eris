use chrono::{DateTime, offset::Utc};
use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

use schema::*;

#[derive(Clone, Debug, Identifiable, Queryable)]
#[table_name="characters"]
#[primary_key(name, game)]
pub struct Character {
    pub char_id: i32,
    pub pinned:  Option<i64>,
    pub mtime:   DateTime<Utc>,
    pub name:    String,
    pub game:    String,
}

#[derive(Clone, Debug, Insertable)]
#[table_name="characters"]
pub struct NewCharacter {
    pub pinned: Option<i64>,
    pub name:   String,
    pub game:   String,
}

#[derive(Clone, Debug, Associations, Identifiable, Insertable, Queryable)]
#[table_name="char_base"]
#[primary_key(char_id)]
#[belongs_to(Character, foreign_key="char_id")]
pub struct CharValues {
    pub char_id:  i32,
    pub cur_hp:   i32,
    pub max_hp:   i32,
    pub cur_rp:   i32,
    pub max_rp:   i32,
    pub cur_fp:   i32,
    pub max_fp:   i32,
    pub cur_lfp:  i32,
    pub max_lfp:  i32,
    pub cur_sp:   i32,
    pub max_sp:   i32,
    pub cur_lsp:  i32,
    pub max_lsp:  i32,
    pub cur_ip:   i32,
    pub max_ip:   i32,
    pub xp:       i32,
}

#[derive(Clone, Debug, Associations, Identifiable, Insertable, Queryable)]
#[table_name="char_notes"]
#[primary_key(char_id, key)]
#[belongs_to(Character, foreign_key="char_id")]
pub struct CharNote {
    pub char_id: i32,
    pub key: String,
    pub value: String
}

#[derive(Clone, Debug, Associations, Identifiable, Insertable, Queryable)]
#[table_name="char_points"]
#[primary_key(char_id, key)]
#[belongs_to(Character, foreign_key="char_id")]
pub struct CharPoint {
    pub char_id: i32,
    pub maximum: i32,
    pub value: i32,
    pub key: String,
}

#[derive(Clone, Debug, Identifiable, Insertable, Queryable)]
#[table_name="guilds"]
#[primary_key(guild_id)]
pub struct GuildEntry {
    pub guild_id: i64,
    pub prefix: Option<String>
}

pub fn establish_connection() -> PgConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL not found in environment");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}
