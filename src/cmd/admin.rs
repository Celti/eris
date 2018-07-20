use crate::types::*;
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
    let map = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db = handle.get()?;
    let guild_id = msg.guild_id().unwrap().0 as i64;
    let prefix = args.full().to_string();

    db.execute("INSERT INTO guilds (guild_id, prefix) VALUES (?1, ?2)
        ON CONFLICT (guild_id) DO UPDATE SET prefix=excluded.prefix",
        &[&guild_id, &prefix])?;

    msg.reply(&format!("Changed prefix to `{}`.",  prefix))?;
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
