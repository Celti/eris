use chrono::{offset::Utc, DateTime};
use log::{log, warn};
use rustbreak::{deser::Bincode, FileDatabase};
use serde_derive::{Deserialize, Serialize};
use serenity::framework::standard::CommandError;
use serenity::model::prelude::*;
use serenity::prelude::{Client, Context, Mutex};
use std::{collections::BTreeMap, sync::Arc};
use typemap::Key;

pub struct DiceCache;
impl Key for DiceCache {
    type Value = BTreeMap<MessageId, String>;
}

pub struct MemoryCache;
impl Key for MemoryCache {
    type Value = BTreeMap<String, MemoryEntry>;
}

pub struct PrefixCache;
impl Key for PrefixCache {
    type Value = BTreeMap<GuildId, String>;
}

struct Persistent;
impl Key for Persistent {
    type Value = FileDatabase<Store, Bincode>;
}

pub struct ShardManager;
impl Key for ShardManager {
    type Value = Arc<Mutex<serenity::client::bridge::gateway::ShardManager>>;
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Store {
    pub prefix: <PrefixCache as Key>::Value,
    pub memory: <MemoryCache as Key>::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MemoryEntry {
    pub content: String,
    pub submitter: String,
    pub timestamp: DateTime<Utc>,
}

pub fn init(client: &mut Client) -> Result<(), failure::Error> {
    let mut map = client.data.lock();
    let db = <Persistent as Key>::Value::from_path("eris.db", Store::default())?;

    if db.load().is_err() {
        warn!("database unreadable or empty, starting fresh");
        std::fs::copy("eris.db", "eris.db~")?;
        db.save()?;
    }

    db.read(|db| {
        map.insert::<ShardManager>(Arc::clone(&client.shard_manager));
        map.insert::<DiceCache>(<DiceCache as Key>::Value::default());
        map.insert::<PrefixCache>(db.prefix.clone());
        map.insert::<MemoryCache>(db.memory.clone());
    })?;

    map.insert::<Persistent>(db);

    Ok(())
}

pub fn sync(ctx: &mut Context, _msg: &Message, _res: &Result<(), CommandError>) {
    let map = ctx.data.lock();
    let db = map.get::<Persistent>().unwrap();

    db.write(|db| {
        db.prefix = map.get::<PrefixCache>().unwrap().clone();
        db.memory = map.get::<MemoryCache>().unwrap().clone();
    }).unwrap();

    db.save().unwrap();
}
