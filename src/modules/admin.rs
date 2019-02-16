use crate::db::DB;
use crate::model::{Prefix, PrefixCache, SerenityShardManager};
use serenity::model::misc::Mentionable;
use serenity::model::permissions::Permissions;
use serenity::CACHE;
use std::io::{Seek, SeekFrom, Write};

cmd!(CacheDump(_ctx, msg)
     aliases: ["cache_dump", "dump_cache"],
     desc: "Dumps the current shard event cache.",
     dm_only: true,
     owners_only: true,
{
    let mut buf = tempfile::tempfile()?;

    write!(&mut buf, "{:#?}", *CACHE.read())?;
    buf.seek(SeekFrom::Start(0))?;

    let filename = format!("eris-cache-{}.log", msg.id.created_at());
    let content = format!("Cache dump for {}.", msg.id.created_at());

    msg.channel_id.send_files(Some((&buf, &*filename)), |m| m.reactions(Some('❌')).content(content))?;
});

cmd!(ChangeGame(ctx, msg, args)
     aliases: ["playing", "play"],
     desc: "Set the currently displayed game tag.",
     min_args: 1,
     owners_only: true,
{
    ctx.set_game(args.full());
    msg.reply("Game set.")?;
});

cmd!(ChangeNick(_ctx, msg, args)
     aliases: ["nick"],
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
     aliases: ["prefix"],
     desc: "Change the command prefix for the current guild or channel.",
     required_permissions: Permissions::ADMINISTRATOR,
{
    let new = {
        if msg.guild_id.is_none() || args.starts_with("channel") {
            Prefix {
                id:     -(msg.channel_id.into():i64),
                prefix: args.trim_start_matches("channel").trim().to_string(),
            }
        } else {
            Prefix {
                id:     msg.guild_id.unwrap().into():i64,
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
     aliases: ["topic"],
     desc: "Change the current channel topic.",
     guild_only: true,
     required_permissions: Permissions::MANAGE_CHANNELS,
{
    let new = args.full().trim();
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
     aliases: ["quit"],
     desc: "Disconnect the bot from Discord.",
     owners_only: true,
{
    let map = ctx.data.lock();
    let shard_manager = map.get::<SerenityShardManager>().unwrap();

    err_log!(msg.channel_id.say("Goodnight, everybody!"));
    shard_manager.lock().shutdown_all();
});

grp![CacheDump, ChangeGame, ChangeNick, ChangePrefix, ChangeTopic, Quit];
