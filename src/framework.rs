#![allow(clippy::needless_pass_by_value)]

use crate::db::DB;
use crate::model::PrefixCache;

use serenity::framework::standard::{
    help_commands,
    Args,
    CommandError,
    CommandGroup,
    DispatchError,
    //HelpBehaviour,
    HelpOptions,
    StandardFramework,
};

use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::id::UserId;

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

pub struct Framework;
impl Framework {
    pub fn standard(owners: HashSet<UserId>) -> StandardFramework {
        StandardFramework::new()
            .configure(|c| {
                c.allow_dm(true)
                    .allow_whitespace(false)
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
            })
            .after(after)
            //.before(before)
            .help(help)
            .on_dispatch_error(on_dispatch_error)
            .unrecognised_command(unrecognised_command)
            .group("Admin", crate::modules::admin::commands)
            .group("Dice", crate::modules::dice::commands)
            .group("GURPS", crate::modules::gurps::commands)
            .group("Logger", crate::modules::logger::commands)
            .group("Memory", crate::modules::memory::commands)
            .group("Random", crate::modules::random::commands)
            .group("Tools", crate::modules::util::commands)
            .group("Toys", crate::modules::toys::commands)
            .group("Character Tracker", |g| crate::modules::chartrack::commands(g).prefix("ct"))
    }
}

fn after(_: &mut Context, _: &Message, cmd: &str, res: Result<(), CommandError>) {
    match res {
        Ok(()) => log::info!("Successfully processed command `{}`", cmd),
        Err(e) => log::warn!("Error processing command `{}`: {:?}", cmd, e),
    }
}

fn dynamic_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    let mut map = ctx.data.lock();
    let cache = map
        .entry::<PrefixCache>()
        .or_insert_with(|| DB.get_prefixes().unwrap());

    let channel_prefix = cache
        .get(&(-(msg.channel_id.into(): i64)))
        .filter(|s| !s.is_empty());
    let guild_prefix = || {
        msg.guild_id
            .and_then(|i| cache.get(&(i.into(): i64)).filter(|s| !s.is_empty()))
    };

    channel_prefix.or_else(guild_prefix).cloned()
}

fn help(
    _ctx: &mut Context,
    msg: &Message,
    opts: &HelpOptions,
    grps: HashMap<String, Arc<CommandGroup>>,
    args: &Args,
) -> Result<(), CommandError> {
    help_commands::with_embeds(_ctx, msg, opts, grps, args)
}

fn on_dispatch_error(_: Context, msg: Message, err: DispatchError) {
    match err {
        DispatchError::OnlyForDM => {
            reply!(msg, "This command is only available in DMs.");
        }

        DispatchError::OnlyForGuilds => {
            reply!(msg, "This command is only available in servers.");
        }

        DispatchError::RateLimited(t) => {
            reply!(msg, "Ratelimited; please wait at least {} seconds.", t);
        }

        DispatchError::NotEnoughArguments { min: m, given: n } => {
            reply!(
                msg,
                "This command takes at least {} arguments (gave {}).",
                m,
                n
            );
        }

        DispatchError::TooManyArguments { max: m, given: n } => {
            reply!(
                msg,
                "This command takes at most {} arguments (gave {}).",
                m,
                n
            );
        }

        _ => {
            log::info!("Command not executed: {:?}", err);
        }
    }
}

fn unrecognised_command(_: &mut Context, msg: &Message, name: &str) {
    match DB.get_bareword(name) {
        Err(diesel::result::Error::NotFound) => (),
        Err(err) => log::warn!("[{}:{}] {:?}", line!(), column!(), err),
        Ok(def) => {
            if def.embedded {
                err_log!(msg
                    .channel_id
                    .send_message(|m| m.embed(|e| e.image(&def.definition))));
            } else {
                err_log!(msg.channel_id.say(&def.definition));
            }
        }
    }
}
