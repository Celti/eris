use crate::types::*;

use diesel::PgConnection;
use failure::Error;
use r2d2_diesel::ConnectionManager;
use serenity::model::channel::Message;
use serenity::prelude::{Client, Context};
use typemap::Key;

pub fn init(client: &mut Client) -> Result<(), Error> {
    let manager = ConnectionManager::<PgConnection>::new(std::env::var("DATABASE_URL")?);
    let pool = r2d2::Pool::new(manager)?;
    let db = pool.get()?;

    let mut map = client.data.lock();

    map.insert::<DatabaseHandle>(pool);
    map.insert::<ShardManager>(client.shard_manager.clone());

    map.insert::<PrefixCache>({
        use diesel::prelude::*;
        use crate::schema::prefixes::dsl::*;

        let mut cache = <PrefixCache as Key>::Value::default();

        for row in prefixes.load::<PrefixEntry>(&*db)? {
            if row.prefix.is_some() {
                cache.insert(row.id, row.prefix.unwrap());
            }
        }

        cache
    });

    map.insert::<DiceCache>(<DiceCache as Key>::Value::default());
    map.insert::<MemoryCache>(<MemoryCache as Key>::Value::default());

    Ok(())
}

pub fn dynamic_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    let map   = ctx.data.lock();
    let cache = map.get::<PrefixCache>()?;

    msg.channel().and_then(|c| Some(c.id()))
        .and_then(|i| cache.get(&-(i.0 as i64)).filter(|s| !s.is_empty()))
    .or_else(|| msg.guild_id
        .and_then(|i| cache.get(&(i.0 as i64)).filter(|s| !s.is_empty())))
    .cloned()
}
