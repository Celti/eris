use crate::db::DB;
use crate::model::{Prefix, PrefixCache, SerenityShardManager, Snowflake};
use serenity::framework::standard::CreateGroup;
use serenity::model::misc::Mentionable;
use serenity::model::permissions::Permissions;

cmd!(CacheDump(_ctx, msg)
     desc: "Dumps the current shard event cache.",
     dm_only: true,
     owners_only: true,
{
    serenity::utils::with_cache(|cache| reply!(msg, "CACHE DUMP: {:?}", cache));
});

cmd!(ChangeGame(ctx, msg, args)
    aliases: ["play"],
    desc: "Set the currently displayed game tag.",
    min_args: 1,
    owners_only: true,
{
    ctx.set_game(args.full());
    msg.reply("Game set.")?;
});

cmd!(ChangeNick(_ctx, msg, args)
    desc: "Change Eris's nickname on the current guild.",
    guild_only: true,
    required_permissions: Permissions::ADMINISTRATOR,
{
    let guild = msg.guild_id.unwrap();
    let nick = args.full();

    guild.edit_nickname(match nick {
        n if n.is_empty() => None,
        n => Some(n)
    })?;
});

cmd!(ChangePrefix(ctx, msg, args)
     desc: "Change the command prefix for the current guild or channel.",
     required_permissions: Permissions::ADMINISTRATOR,
{
    let new = {
        if msg.guild_id.is_none() || args.starts_with("channel") {
            Prefix {
                id:     -msg.channel_id.to_i64(),
                prefix: args.trim_left_matches("channel").trim().to_string(),
            }
        } else {
            Prefix {
                id:     msg.guild_id.unwrap().to_i64(),
                prefix: args.trim().to_string(),
            }
        }
    };

    DB.upsert_prefix(&new)?;

    let old = ctx.data.lock()
        .entry::<PrefixCache>()
        .or_insert_with(|| DB.get_prefixes().unwrap())
        .insert(new.id, new.prefix.clone());

    if let Some(old) = old.filter(|s| !s.is_empty()) {
        if new.prefix.is_empty() {
            reply!(msg, "Changed prefix from `{}` to default.", old);
        } else {
            reply!(msg, "Changed prefix from `{}` to `{}`.", old, new.prefix);
        }
    } else if new.prefix.is_empty() {
        err_log!(msg.reply("The prefix has not been changed."));
    } else {
        reply!(msg, "Changed prefix to `{}`.",  new.prefix);
    }
});

cmd!(ChangeTopic(_ctx, msg, args)
    desc: "Change the current channel topic.",
    guild_only: true,
    required_permissions: Permissions::MANAGE_CHANNELS,
{
    let new = args.trim();
    let old = msg.channel_id.to_channel_cached()
        .and_then(|c| c.guild())
        .and_then(|g| g.read().topic.clone());
    let mention = msg.channel_id.mention();

    msg.channel_id.edit(|c| c.topic(&new))?;

    if let Some(old) = old.filter(|s| !s.is_empty()) {
        if new.is_empty() {
            say!(msg.channel_id, "Unset topic for {}.", mention);
        } else {
            say!(msg.channel_id, "Changed topic for {} from `{}` to `{}`.", mention, old, new);
        }
    } else {
        say!(msg.channel_id, "Set topic for {} to `{}`.", mention, new);
    }
});

cmd!(Quit(ctx, msg)
    desc: "Disconnect the bot from Discord.",
    owners_only: true,
{
    let map = ctx.data.lock();
    let shard_manager = map.get::<SerenityShardManager>().unwrap();

    err_log!(msg.channel_id.say("Goodnight, everybody!"));
    shard_manager.lock().shutdown_all();
});

pub fn commands(g: CreateGroup) -> CreateGroup {
    g.cmd("dump_cache", CacheDump::new())
     .cmd("playing",    ChangeGame::new())
     .cmd("nick",       ChangeNick::new())
     .cmd("prefix",     ChangePrefix::new())
     .cmd("topic",      ChangeTopic::new())
     .cmd("quit",       Quit::new())
}
