use {db, diesel};
use ext::dice::DiceVec;
use parking_lot::Mutex;
use serenity::client::{bridge::gateway, Client};
use serenity::model::id::{GuildId, MessageId};
use std::collections::BTreeMap;
use std::sync::Arc;
use typemap::Key;

// ShareMap keys
pub struct DiceCache;
impl Key for DiceCache {
    type Value = BTreeMap<MessageId, DiceVec>;
}

pub struct PrefixCache;
impl Key for PrefixCache {
    type Value = BTreeMap<GuildId, String>;
}

pub struct ShardManager;
impl Key for ShardManager {
    type Value = Arc<Mutex<gateway::ShardManager>>;
}

pub struct DatabaseConnection;
impl Key for DatabaseConnection {
    type Value = Arc<Mutex<diesel::pg::PgConnection>>;
}

pub fn init(client: &mut Client) {
    let mut data = client.data.lock();
    let db = db::establish_connection();

    data.insert::<PrefixCache>({
        use db::GuildEntry;
        use diesel::prelude::*;
        use schema::guilds::dsl::*;

        let mut map = BTreeMap::default();
        let query = guilds.load::<GuildEntry>(&db)
            .expect("Error querying database");

        for row in query {
            if row.prefix.is_some() {
                map.insert(GuildId(row.guild_id as u64), row.prefix.unwrap());
            }
        }

        map
    });

    data.insert::<DatabaseConnection>(Arc::new(Mutex::new(db)));
    data.insert::<DiceCache>(BTreeMap::default());
    data.insert::<ShardManager>(Arc::clone(&client.shard_manager));
}
