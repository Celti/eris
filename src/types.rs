use crate::schema::*;

use chrono::{DateTime, Utc};
use diesel::PgConnection;
use r2d2_diesel::ConnectionManager;
use serenity::model::id::{ChannelId, GuildId, MessageId, UserId};
use serenity::prelude::Mutex;
use std::{collections::HashMap, sync::Arc};

pub use serenity::prelude::Mentionable;
pub use typemap::Key;

pub type DB = diesel::pg::Pg;

pub struct DiceCache;
impl Key for DiceCache {
    type Value = HashMap<MessageId, String>;
}

pub struct PrefixCache;
impl Key for PrefixCache {
    type Value = HashMap<GuildId, String>;
}

pub struct MemoryCache;
impl Key for MemoryCache {
    type Value = HashMap<ChannelId, CurrentMemory>;
}

pub struct DatabaseHandle;
impl Key for DatabaseHandle {
    type Value = r2d2::Pool<ConnectionManager<PgConnection>>;
}

pub struct ShardManager;
impl Key for ShardManager {
    type Value = Arc<Mutex<serenity::client::bridge::gateway::ShardManager>>;
}

#[derive(Clone, Debug, Queryable)]
pub struct GuildEntry {
    pub guild_id: i64, // GuildId
    pub prefix:   Option<String>
}

#[derive(Clone, Debug, Insertable, AsChangeset)]
#[table_name="guilds"]
#[primary_key(guild_id)]
pub struct NewGuildEntry<'a> {
    pub guild_id: i64, // GuildId
    pub prefix:   &'a str,
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Queryable)]
pub struct KeywordEntry {
    pub keyword: String,
    pub owner:   Option<i64>, // Option<UserId>
    pub shuffle: bool,
    pub protect: bool,
    pub hidden:  bool,
    pub bare:    bool,
}

#[cfg_attr(feature = "cargo-clippy", allow(option_option))]
#[derive(Clone, Debug, Default, PartialEq, PartialOrd, AsChangeset)]
#[table_name="keywords"]
#[primary_key(keyword)]
pub struct NewKeywordEntry {
    pub owner:   Option<Option<i64>>, // Option<UserId>
    pub shuffle: Option<bool>,
    pub protect: Option<bool>,
    pub hidden:  Option<bool>,
    pub bare:    Option<bool>,
}

impl NewKeywordEntry {
    pub fn shuffle(&mut self, value: bool) {
        self.shuffle = Some(value);
    }

    pub fn protect(&mut self, value: bool, owner: i64) {
        self.protect = Some(value);

        if value {
            self.owner = Some(Some(owner));
        } else {
            self.owner = Some(None);
        }
    }

    pub fn hidden(&mut self, value: bool, owner: i64) {
        self.hidden = Some(value);

        if value {
            self.owner = Some(Some(owner));
        } else {
            self.owner = Some(None);
        }
    }

    pub fn bare(&mut self, value: bool, owner: i64) {
        self.bare = Some(value);

        if value {
            self.owner = Some(Some(owner));
        } else {
            self.owner = Some(None);
        }
    }
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Queryable)]
pub struct DefinitionEntry {
    pub keyword:    String,
    pub definition: String,
    pub submitter:  i64, // UserId
    pub timestamp:  DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Identifiable, Insertable, AsChangeset)]
#[table_name="definitions"]
#[primary_key(keyword, definition)]
pub struct NewDefinitionEntry<'a> {
    pub keyword:    &'a str,
    pub definition: &'a str,
    pub submitter:  i64, // UserId
}

#[derive(Clone, Debug, Default)]
pub struct CurrentMemory {
    pub idx: usize,
    pub key: KeywordEntry,
    pub def: Vec<DefinitionEntry>,
}

impl CurrentMemory {
    pub fn content(&self) -> String {
        format!("{} {}", self.def[self.idx].keyword, self.def[self.idx].definition)
    }

    pub fn definition(&self) -> String {
        self.def[self.idx].definition.to_string()
    }

    pub fn details(&self) -> String {
        format!("{} ({}/{}) submitted by {} at {}.",
            self.def[self.idx].keyword,
            self.idx + 1,
            self.def.len(),
            UserId(self.def[self.idx].submitter as u64).mention(),
            self.def[self.idx].timestamp,
        )
    }

    pub fn next(&mut self) {
        if self.idx == self.def.len() - 1 {
            self.idx = 0;
        } else {
            self.idx += 1;
        }
    }

    pub fn prev(&mut self) {
        if self.idx == 0 {
            self.idx = self.def.len() - 1;
        } else {
            self.idx -= 1;
        }
    }
}
