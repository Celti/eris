use crate::types::*;
use serenity::command;

command!(change_nick(_ctx, msg, args) {
    let guild = msg.guild_id.unwrap();
    let nick = args.full();

    guild.edit_nickname(match nick {
        n if n.is_empty() => None,
        n => Some(n)
    })?;
});

command!(change_prefix(ctx, msg, args) {
    use diesel::{prelude::*, pg::upsert::excluded};
    use crate::schema::*;

    let mut map = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap().clone();
    let cache = map.get_mut::<PrefixCache>().unwrap();

    let new =
        if msg.guild_id.is_none() || args.starts_with("channel") {
            NewPrefixEntry {
                id: -(msg.channel().unwrap().id().0 as i64),
                prefix: args.trim_left_matches("channel").trim(),
            }
        } else {
            NewPrefixEntry {
                id: msg.guild_id.unwrap().0 as i64,
                prefix: args.trim(),
            }
        };

    diesel::insert_into(prefixes::table)
        .values(&new)
        .on_conflict(prefixes::id)
        .do_update()
        .set(prefixes::prefix.eq(excluded(prefixes::prefix)))
        .execute(&*handle.get()?)?;

    let old = cache.insert(new.id, new.prefix.to_string());

    if let Some(old) = old.filter(|s| !s.is_empty()) {
        if new.prefix.is_empty() {
            msg.reply(&format!("Changed prefix from `{}` to default.", old))?;
        } else {
            msg.reply(&format!("Changed prefix from `{}` to `{}`.", old, new.prefix))?;
        }
    } else if new.prefix.is_empty() {
        msg.reply("The prefix has not been changed.")?;
    } else {
        msg.reply(&format!("Changed prefix to `{}`.",  new.prefix))?;
    }
});

command!(change_topic(_ctx, msg, args) {
    let new = args.full();

    if let Some(ref old) = msg.channel_id.to_channel()?.guild().unwrap().read().topic {
        msg.reply(&format!("Changed topic for {} from `{}` to `{}`.", msg.channel_id.mention(), old, new))?;
    } else {
        msg.reply(&format!("Set topic for {} to `{}`.", msg.channel_id.mention(), new))?;
    }

    msg.channel_id.edit(|c| c.topic(&new))?;
});

command!(set_playing(ctx, msg, args) {
    ctx.set_game(args.full());
    msg.reply("Game set.")?;
});

command!(quit(ctx, msg) {
    let map = ctx.data.lock();
    let shard_manager = map.get::<ShardManager>().unwrap();

    msg.channel_id.say("Goodnight, everybody!")?;
    shard_manager.lock().shutdown_all();
});
