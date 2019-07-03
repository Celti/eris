#![allow(clippy::needless_pass_by_value)]

use crate::db::{DynamicPrefix, Memory};
use crate::model::PrefixCache;
use crate::model::PermissionsExt;
use maplit::hashset;
use serenity::client::Context;
use serenity::framework::standard::*;
use serenity::framework::standard::macros::help;
use serenity::model::channel::Message;
use serenity::model::id::UserId;
use std::collections::HashSet;

pub fn new(owner_id: UserId, bot_id: UserId) -> StandardFramework {
    StandardFramework::new()
        .after(after)
        .before(before)
        .configure(|c| { c
            .allow_dm(true)
            .case_insensitivity(true)
            .delimiters(vec![Delimiter::Multiple(", or ".into()),
                             Delimiter::Multiple(", ".into()),
                             Delimiter::Single(','),
                             Delimiter::Multiple(" or ".into()),
                             Delimiter::Single(' ')])
            .disabled_commands(hashset![])
            .dynamic_prefixes(vec![Box::new(dynamic_prefix)])
            .ignore_bots(true)
            .ignore_webhooks(true)
            .no_dm_prefix(true)
            .on_mention(Some(bot_id))
            .owners(hashset![owner_id])
            .prefixes(Vec::<String>::new())
            .with_whitespace((true, true, true))})
        .group(&crate::modules::admin::ADMIN_GROUP)
        .group(&crate::modules::chartrack::TRACKER_GROUP)
        .group(&crate::modules::dice::DICE_GROUP)
        .group(&crate::modules::gurps::GURPS_GROUP)
        .group(&crate::modules::memory::MEMORY_GROUP)
        .group(&crate::modules::random::RANDOM_GROUP)
        .group(&crate::modules::toys::TOYS_GROUP)
        .group(&crate::modules::util::UTIL_GROUP)
        .help(&HELP)
        .on_dispatch_error(on_dispatch_error)
        .unrecognised_command(unrecognised_command)
}

fn after(_: &mut Context, _: &Message, cmd: &str, res: Result<(), CommandError>) {
    match res {
        Ok(()) => log::info!("Successfully processed command `{}`", cmd),
        Err(e) => log::warn!("Error processing command `{}`: {:?}", cmd, e),
    }
}

fn before(_: &mut Context, _: &Message, cmd: &str) -> bool {
    log::info!("Processing command `{}`", cmd);
    true
}

// TODO dynamic_prefix_by_guild(), dynamic_prefix_by_channel(), dynamic_prefix_by_user()
fn dynamic_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    let mut map = ctx.data.write();
    let cache = map
        .entry::<PrefixCache>()
        .or_insert_with(|| DynamicPrefix::get().unwrap());

    let channel_prefix = cache
        .get(&{ let i:i64 = msg.channel_id.into(); -i})
        .filter(|s| !s.is_empty());
    let guild_prefix = || {
        msg.guild_id
            .and_then(|i| cache.get(&i.into()).filter(|s| !s.is_empty()))
    };

    channel_prefix.or_else(guild_prefix).cloned()
}

#[help]
fn help(
    context: &mut Context,
    msg: &Message,
    args: Args,
    options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>
) -> CommandResult {
    help_commands::with_embeds(context, msg, args, options, groups, owners)
}

fn on_dispatch_error(ctx: &mut Context, msg: &Message, err: DispatchError) {
    use DispatchError::*;
    match err {
        NotEnoughArguments { min, given } => {
            reply!(ctx, msg, "This command takes at least {} arguments ({} given).", min, given);
        }
        TooManyArguments { max, given } => {
            reply!(ctx, msg, "This command takes at most {} arguments ({} given).", max, given);
        }
        LackingPermissions(p) => reply!(ctx, msg, "Permission denied. Required: {}.", p.pretty()),
        LackingRole           => reply!(ctx, msg, "This command is restricted."),
        OnlyForDM             => reply!(ctx, msg, "This command is restricted to DMs."),
        OnlyForGuilds         => reply!(ctx, msg, "This command is restricted to servers."),
        OnlyForOwners         => reply!(ctx, msg, "This command is restricted to the bot owner."),
        Ratelimited(t)        => reply!(ctx, msg, "Command ratelimited for {} more seconds.", t),
        CheckFailed(c, r)     => log::info!("Check failed: {}: {:?}", c, r),
        CommandDisabled(name) => log::info!("Command disabled: {}", name),
        IgnoredBot            => log::info!("Ignoring bot user."),
        WebhookAuthor         => log::info!("Ignoring webhook."),
        other                 => log::trace!("Unexpected dispatch error: {:?}", other),
    }
}

fn unrecognised_command(ctx: &mut Context, msg: &Message, name: &str) {
    match Memory::get_bareword(name) {
        Err(diesel::result::Error::NotFound) => (),
        Err(err) => log::warn!("[{}:{}] {:?}", line!(), column!(), err),
        Ok(def) => {
            if def.embedded {
                err_log!(msg
                    .channel_id
                    .send_message(&ctx, |m| m.embed(|e| e.image(&def.definition))));
            } else {
                say!(ctx, msg, "{}", def.definition);
            }
        }
    }
}
