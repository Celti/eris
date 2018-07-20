use crate::types::*;
use failure::Error;
use serenity::model::prelude::Message;
use serenity::prelude::{Client, Context};
use typemap::Key;

pub fn init(client: &mut Client) -> Result<(), Error> {
    let manager = r2d2_sqlite::SqliteConnectionManager::file(&std::env::var("DATABASE_FILE")?);
    let pool = r2d2::Pool::new(manager)?;

    let mut map = client.data.lock();

    map.insert::<DatabaseHandle>(pool);
    map.insert::<ShardManager>(client.shard_manager.clone());

    map.insert::<DiceCache>(<DiceCache as Key>::Value::default());
    map.insert::<MemoryCache>(<MemoryCache as Key>::Value::default());
    map.insert::<PrefixCache>(<PrefixCache as Key>::Value::default());

    Ok(())
}

pub fn dynamic_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    let map = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>()?;
    let db = handle.try_get()?;
    let guild_id = msg.guild_id()?.0 as i64;

    db.query_row("SELECT prefix FROM guilds WHERE guild_id=?",
                 &[&guild_id], |r| r.get(0)).ok()
}
