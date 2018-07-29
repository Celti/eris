// FIXME use_extern_macros
// use serenity::command;

use crate::types::*;
use serenity::model::id::GuildId;

command!(change_nick(_ctx, msg, args) {
    let guild = msg.guild_id.unwrap();
    let nick = args.full();

    guild.edit_nickname(match nick {
        n if n.is_empty() => None,
        n => Some(n)
    })?;
});

command!(change_guild_prefix(ctx, msg, args) {
    use crate::schema::guilds::dsl::*;
    use diesel::{prelude::*, pg::upsert::excluded};

    let mut map = ctx.data.lock();

    let new = NewGuildEntry {
        guild_id: msg.guild_id.unwrap().0 as i64,
        prefix:   args.full(),
    };

    let handle = map.get::<DatabaseHandle>().unwrap();
    diesel::insert_into(guilds)
        .values(&new)
        .on_conflict(guild_id)
        .do_update()
        .set(prefix.eq(excluded(prefix)))
        .execute(&*handle.get()?)?;

    let cache = map.get_mut::<PrefixCache>().unwrap();
    if let Some(old) = cache.insert(GuildId(new.guild_id as u64), new.prefix.to_string()) {
        msg.reply(&format!("Changed prefix from `{}` to `{}`.", old, new.prefix))?;
    } else {
        msg.reply(&format!("Changed prefix to `{}`.",  new.prefix))?;
    }
});

command!(change_topic(_ctx, msg, args) {
    let new = args.full();

    if let Some(ref old) = msg.channel_id.get()?.guild().unwrap().read().topic {
        msg.reply(&format!("Changed topic for {} from `{}` to `{}`.", msg.channel_id.mention(), old, new))?;
    } else {
        msg.reply(&format!("Set topic for {} to `{}`.", msg.channel_id.mention(), new))?;
    }

    msg.channel_id.edit(|c| c.topic(&new))?;
});

command!(set_playing(ctx, msg, args) {
    ctx.set_game_name(&args.full());
    msg.reply("Game set.")?;
});

command!(quit(ctx, msg) {
    let map = ctx.data.lock();
    let shard_manager = map.get::<ShardManager>().unwrap();

    msg.channel_id.say("Goodnight, everybody!")?;
    shard_manager.lock().shutdown_all();
});
