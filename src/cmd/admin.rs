use crate::key::{PrefixCache, ShardManager};
use log::{info, log};
use serenity::command;

command!(change_nick(_ctx, msg, args) {
    let guild = msg.guild_id().unwrap();
    let nick = args.full();

    guild.edit_nickname(match nick {
        n if n.is_empty() => None,
        n => Some(n)
    })?;
});

command!(change_guild_prefix(ctx, msg, args) {
    let mut map = ctx.data.lock();

    let cache = map.get_mut::<PrefixCache>().unwrap();
    let guild = msg.guild_id().unwrap();
    let new   = args.full().to_string();

    if let Some(old) = cache.insert(guild, new.clone()) {
        info!("PREFIX: old prefix is {}", old);
        msg.reply(&format!("Changed prefix from `{}` to `{}`.", old, new))?;
    } else {
        msg.reply(&format!("Changed prefix from default to `{}`.",  new))?;
    }
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
