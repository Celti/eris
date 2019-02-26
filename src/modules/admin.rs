use crate::db::DynamicPrefix as DB;
use crate::model::{Prefix, PrefixCache, SerenityShardManager};
use serenity::model::channel::Channel;
use serenity::model::gateway::Activity;
use serenity::model::misc::Mentionable;
use std::io::{Seek, SeekFrom, Write};

use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;

#[command]
#[aliases(dump_cache)]
#[description = "Dumps the current shard event cache."]
#[only_in(dms)]
#[owners_only(true)]
fn cache_dump(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut buf = tempfile::tempfile()?;

    write!(&mut buf, "{:#?}", *ctx.cache.read())?;
    buf.seek(SeekFrom::Start(0))?;

    let filename = format!("eris-cache-{}.log", msg.id.created_at());
    let content = format!("Cache dump for {}.", msg.id.created_at());

    msg.channel_id.send_files(&ctx.http, Some((&buf, &*filename)), |m| m.reactions(Some('❌')).content(content))?;

    Ok(())
}

#[command]
#[aliases(play)]
#[description("Set the currently displayed activity.")]
#[min_args(1)]
#[owners_only(true)]
fn playing(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    ctx.set_activity(Activity::playing(args.message()));
    reply!(ctx, msg, "Game set.");

    Ok(())
}

#[command]
#[description("Change Eris's nickname on the current guild.")]
#[only_in(guilds)]
#[required_permissions(ADMINISTRATOR)]
fn nick(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let guild = msg.guild_id.unwrap();
    let nick = args.message();

    guild.edit_nickname(&ctx.http, match nick {
        n if n.is_empty() => None,
        n => Some(n)
    })?;

    Ok(())
}

#[command]
#[description("Change the command prefix for the current guild or channel.")]
#[required_permissions(ADMINISTRATOR)]
fn prefix(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let new = {
        if msg.guild_id.is_none() || args.message().starts_with("channel") {
            Prefix {
                id:     { let i:i64 = msg.channel_id.into(); -i },
                prefix: args.message().trim_start_matches("channel").trim().to_string(),
            }
        } else {
            Prefix {
                id:     msg.guild_id.unwrap().into(),
                prefix: args.message().trim().to_string(),
            }
        }
    };

    DB::set(&new)?;

    let old = ctx.data.write()
        .entry::<PrefixCache>()
        .or_insert_with(|| DB::get().unwrap())
        .insert(new.id, new.prefix.clone());

    if let Some(old) = old.filter(|s| !s.is_empty()) {
        if new.prefix.is_empty() {
            reply!(ctx, msg, "Changed prefix from `{}` to default.", old);
        } else {
            reply!(ctx, msg, "Changed prefix from `{}` to `{}`.", old, new.prefix);
        }
    } else if new.prefix.is_empty() {
        reply!(ctx, msg, "The prefix has not been changed.");
    } else {
        reply!(ctx, msg, "Changed prefix to `{}`.",  new.prefix);
    }

    Ok(())
}

#[command]
#[description("Change the current channel topic.")]
#[only_in(guilds)]
#[required_permissions(MANAGE_CHANNELS)]
fn topic(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let new = args.message().trim();
    let old = msg.channel_id.to_channel_cached(&ctx.cache)
        .and_then(Channel::guild)
        .and_then(|g| g.read().topic.clone());
    let mention = msg.channel_id.mention();

    msg.channel_id.edit(&ctx.http, |c| c.topic(&new))?;

    if let Some(old) = old.filter(|s| !s.is_empty()) {
        if new.is_empty() {
            say!(ctx, msg, "Unset topic for {}.", mention);
        } else {
            say!(ctx, msg, "Changed topic for {} from `{}` to `{}`.", mention, old, new);
        }
    } else {
        say!(ctx, msg, "Set topic for {} to `{}`.", mention, new);
    }

    Ok(())
}

#[command]
#[description("Disconnect the bot from Discord.")]
#[owners_only(true)]
fn quit(ctx: &mut Context, msg: &Message) -> CommandResult {
    let map = ctx.data.read();
    let shard_manager = map.get::<SerenityShardManager>().unwrap();

    say!(ctx, msg, "Goodnight, everybody!");
    shard_manager.lock().shutdown_all();

    Ok(())
}

group!({
    name: "admin",
    options: {},
    commands: [cache_dump, nick, playing, prefix, quit, topic]
});
