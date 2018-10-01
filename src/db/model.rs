use crate::db::schema::*;

use chrono::{DateTime, Utc};
use diesel::{AsChangeset, Associations, Identifiable, Insertable, Queryable};

#[derive(Clone, Debug, AsChangeset, Identifiable, Insertable, Queryable)]
#[table_name = "prefixes"]
#[primary_key(id)]
pub struct Prefix {
    pub id:     i64,
    pub prefix: String,
}

#[derive(Clone, Debug, Default, AsChangeset, Identifiable, Insertable, Queryable)]
#[table_name = "keywords"]
#[primary_key(keyword)]
pub struct Keyword {
    pub keyword:  String,
    pub owner:    i64,
    pub bareword: bool,
    pub hidden:   bool,
    pub protect:  bool,
    pub shuffle:  bool,
}

#[derive(Clone, Debug, AsChangeset, Associations, Identifiable, Insertable, Queryable)]
#[belongs_to(Keyword, foreign_key = "keyword")]
#[table_name = "definitions"]
#[primary_key(keyword, definition)]
pub struct Definition {
    pub keyword:    String,
    pub definition: String,
    pub submitter:  i64,
    pub timestamp:  DateTime<Utc>,
    pub embedded:   bool,
}