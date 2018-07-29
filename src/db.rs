use crate::types::*;

use diesel::PgConnection;
use failure::Error;
use r2d2_diesel::ConnectionManager;
use serenity::model::prelude::{GuildId, Message};
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
        use crate::schema::guilds::dsl::*;

        let mut cache = <PrefixCache as Key>::Value::default();

        for row in guilds.load::<GuildEntry>(&*db)? {
            if row.prefix.is_some() {
                cache.insert(GuildId(row.guild_id as u64), row.prefix.unwrap());
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
    cache.get(&msg.guild_id?).filter(|s| !s.is_empty()).cloned()
}
