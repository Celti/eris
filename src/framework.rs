#![allow(clippy::needless_pass_by_value)]

use crate::db::{DynamicPrefix, Memory};
use crate::model::PrefixCache;

use serenity::framework::standard::{
    help_commands::WITH_EMBEDS_HELP_COMMAND,
    CommandError,
    DispatchError,
    StandardFramework,
};

use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::id::UserId;

use std::collections::HashSet;

pub struct Framework;
impl Framework {
    pub fn standard(owners: HashSet<UserId>) -> StandardFramework {
        StandardFramework::new()
            .configure(|c| { c
                .allow_dm(true)
                // .blocked_guilds(hashset!{GuildId(1), GuildId(2)})
                // .blocked_users(hashset!{UserId(1), UserId(2)})
                .case_insensitivity(true)
                // .delimiters(&[", or ", ", ", ",", " or ", " "])
                // .depth(5)
                // .disabled_commands(hashset!{"foo", "fnord"})
                .dynamic_prefix(dynamic_prefix)
                .ignore_bots(true)
                .ignore_webhooks(true)
                .no_dm_prefix(true)
                .on_mention(true)
                .owners(owners)
                .with_whitespace(false)
            })
            .after(after)
            //.before(before)
            .help(&WITH_EMBEDS_HELP_COMMAND)
            .on_dispatch_error(on_dispatch_error)
            .unrecognised_command(unrecognised_command)
            .group(&crate::modules::admin::ADMIN_GROUP)
            .group(&crate::modules::dice::DICE_GROUP)
            .group(&crate::modules::chartrack::TRACKER_GROUP)
            .group(&crate::modules::memory::MEMORY_GROUP)
            .group(&crate::modules::gurps::GURPS_GROUP)
            .group(&crate::modules::random::RANDOM_GROUP)
            .group(&crate::modules::toys::TOYS_GROUP)
            .group(&crate::modules::util::UTIL_GROUP)
    }
}

fn after(_: &mut Context, _: &Message, cmd: &str, res: Result<(), CommandError>) {
    match res {
        Ok(()) => log::info!("Successfully processed command `{}`", cmd),
        Err(e) => log::warn!("Error processing command `{}`: {:?}", cmd, e),
    }
}

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

// #[help]
// fn help(
//     ctx: &mut Context,
//     msg: &Message,
//     args: Args,
//     opts: &'static HelpOptions,
//     groups: &[&'static CommandGroup],
//     owners: HashSet<UserId>,
// ) -> CommandResult {
//     help_commands::with_embeds(ctx, msg, args, opts, groups, owners)
// }

fn on_dispatch_error(ctx: &mut Context, msg: &Message, err: DispatchError) {
    use DispatchError::*;
    match err {
        CheckFailed(c, r)     => log::info!("Check failed: {}: {:?}", c, r),
        CommandDisabled(name) => log::info!("Command disabled: {}", name),
        OnlyForDM             => reply!(ctx, msg, "This command is restricted to DMs."),
        OnlyForGuilds         => reply!(ctx, msg, "This command is restricted to servers."),
        OnlyForOwners         => reply!(ctx, msg, "This command is restricted to the bot owner."),
        LackingRole           => reply!(ctx, msg, "This command is restricted."),
        LackingPermissions(_) => reply!(ctx, msg, "This command is restricted."),
        NotEnoughArguments { min, given } => {
            reply!(ctx, msg, "This command takes at least {} arguments (gave {}).", min, given);
        }
        TooManyArguments { max, given } => {
            reply!(ctx, msg, "This command takes at most {} arguments (gave {}).", max, given);
        }
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
                    .send_message(&ctx.http, |m| m.embed(|e| e.image(&def.definition))));
            } else {
                say!(ctx, msg, "{}", def.definition);
            }
        }
    }
}
