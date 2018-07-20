use serenity::model::prelude::{ChannelId, GuildId, MessageId, UserId};
use serenity::prelude::{Mentionable, Mutex};
use std::{collections::HashMap, sync::Arc};
use serde_derive::Deserialize;
pub use typemap::Key;
use chrono::{DateTime, Utc};

pub type DatabaseConnection = r2d2::PooledConnection<r2d2_sqlite::SqliteConnectionManager>;

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
    type Value = r2d2::Pool<r2d2_sqlite::SqliteConnectionManager>;
}

pub struct ShardManager;
impl Key for ShardManager {
    type Value = Arc<Mutex<serenity::client::bridge::gateway::ShardManager>>;
}

#[derive(Clone, Debug, Default, PartialEq, PartialOrd, Deserialize)]
pub struct KeywordEntry {
    pub keyword: String,
    pub owner:   UserId,
    pub shuffle: bool,
    pub protect: bool,
    pub hidden:  bool,
    pub bare:    bool,
}

#[derive(Clone, Debug, PartialEq, PartialOrd, Deserialize)]
pub struct DefinitionEntry {
    pub keyword:    String,
    pub definition: String,
    pub submitter:  UserId,
    pub timestamp:  DateTime<Utc>,
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

    #[allow(dead_code)]
    pub fn definition(&self) -> String {
        self.def[self.idx].definition.to_string()
    }

    pub fn details(&self) -> String {
        format!("{} ({}/{}) submitted by {} at {}.",
            self.def[self.idx].keyword,
            self.idx + 1,
            self.def.len(),
            self.def[self.idx].submitter.mention(),
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
