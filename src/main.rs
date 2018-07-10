#![feature(rust_2018_preview, use_extern_macros, nll)]

mod cmd;
mod key;
mod util;

use crate::util::cached_display_name;
use exitfailure::ExitFailure;
use failure::SyncFailure;
use log::{debug, error, info, log, warn}; // trace
use serenity::{model::prelude::*, prelude::*};

struct Eris;
impl EventHandler for Eris {
    fn message(&self, _: Context, message: Message) {
        match message.channel_id.get() {
            Ok(Channel::Guild(channel)) => {
                info!(target: "chat",
                    "[{} #{}] {} <{}:{}> {}",
                    channel.read().guild_id.get().unwrap().name,
                    channel.read().name(),
                    message.timestamp,
                    message.author.id,
                    cached_display_name(message.channel_id, message.author.id).unwrap(),
                    message.content
                );
            }
            Ok(Channel::Group(channel)) => {
                info!(target: "chat",
                    "[{}] {} <{}:{}> {}",
                    channel.read().name(),
                    message.timestamp,
                    message.author.id,
                    message.author.name,
                    message.content
                );
            }
            Ok(Channel::Private(channel)) => {
                info!(target: "chat",
                    "[{}] {} <{}:{}> {}",
                    channel.read().name(),
                    message.timestamp,
                    message.author.id,
                    message.author.name,
                    message.content
                );
            }
            Ok(Channel::Category(channel)) => {
                info!(target: "chat",
                    "[{}] {} <{}:{}> {}",
                    channel.read().name(),
                    message.timestamp,
                    message.author.id,
                    message.author.name,
                    message.content
                );
            }
            Err(_) => {
                warn!(target: "chat",
                    "[Unknown Channel] {} <{}:{}> {}",
                    message.timestamp,
                    message.author.id,
                    message.author.name,
                    message.content
                );
            }
        }
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(s) = ready.shard {
            info!("Logged in as '{}' on {}/{}", ready.user.name, s[0], s[1]);
        } else {
            info!("Logged in as '{}'", ready.user.name);
        }

        // TODO get name from persistent store
        ctx.set_game_name("with fire.");
    }

    fn reaction_add(&self, ctx: Context, re: Reaction) {
        use crate::key::DiceCache;
        use serenity::framework::standard::{Args, CommandError};

        // Don't respond to our own reactions.
        if serenity::utils::with_cache(|cache| cache.user.id == re.user_id) {
            return;
        }

        // Reaction matcher.
        match re.emoji {
            // Reroll dice.
            ReactionType::Unicode(ref x) if x == "🎲" => {
                let mut map = ctx.data.lock();
                let cache = map.get_mut::<DiceCache>().unwrap();
                let dice = if let Some(dice) = cache.get(&re.message_id) {
                    dice.clone()
                } else {
                    info!("Die roll is not in message cache.");
                    return;
                };

                let result: Result<(), CommandError> = do catch {
                    let name = re.user_id.mention();
                    let roll = cmd::roll::process_args(Args::new(&dice, &[' '.to_string()]))?;
                    let sent = re.channel_id.send_message(|m| {
                        m.content(format!("**{} rolled:**{}", name, roll))
                            .reactions(vec!['🎲'])
                    })?;

                    re.channel_id
                        .message(re.message_id)
                        .and_then(|m| m.delete_reactions())?;
                    cache.insert(sent.id, dice);
                };

                if let Err(err) = result {
                    error!("error repeating dice roll: {:?}", err);
                }
            }
            // An unconfigured reaction type.
            r => debug!("Unknown ReactionType: {:?}", r),
        }
    }
}

fn main() -> Result<(), ExitFailure> {
    dotenv::dotenv().ok();
    log_panics::init();
    env_logger::init();

    use serenity::framework::standard::{help_commands, StandardFramework};

    let token = std::env::var("DISCORD_TOKEN")?;
    let mut client = Client::new(&token, Eris).map_err(SyncFailure::new)?;

    client.with_framework(
        StandardFramework::new()
            .configure(|c| {
                c
            .allow_dm(true)
            .allow_whitespace(false)
            // .blocked_guilds(hashset!{GuildId(1), GuildId(2)})
            // .blocked_users(hashset!{UserId(1), UserId(2)})
            // .depth(5)
            // .disabled_commands(hashset!{"foo", "fnord"})
            .dynamic_prefix(util::cached_prefix)
            .ignore_bots(true)
            .ignore_webhooks(true)
            .on_mention(true)
            .prefix("$$")
            // .owners(hashset!{info.owner.id})
            // .delimiters(&[", or ", ", ", ",", " or ", " "])
            .case_insensitivity(true)
            })
            .after(|_ctx, _msg, cmd, res| match res {
                Ok(()) => info!("Successfully processed command '{}'", cmd),
                Err(e) => error!("Error processing command '{}': {:?}", cmd, e),
            })
            .on_dispatch_error(|_ctx, msg, err| {
                // TODO match on DispatchError enum and customise responses.
                let _ = msg
                    .channel_id
                    .say(&format!("Could not execute command: {:?}", err));
            })
            .help(help_commands::with_embeds)
            .group("Admin", |g| {
                g.command("nick", |c| {
                    c.desc("Change Eris's nickname on the current guild.")
                        .cmd(cmd::admin::change_nick)
                        .guild_only(true)
                        .required_permissions(Permissions::ADMINISTRATOR)
                }).command("playing", |c| {
                        c.desc("Set the currently displayed game tag.")
                            .cmd(cmd::admin::set_playing)
                            .known_as("play")
                            .min_args(1)
                            .owners_only(true)
                    })
                    .command("prefix", |c| {
                        c.desc("Change Eris's command prefix on the current guild.")
                            .cmd(cmd::admin::change_guild_prefix)
                            .guild_only(true)
                            .required_permissions(Permissions::ADMINISTRATOR)
                            .after(key::sync)
                    })
                    .command("quit", |c| {
                        c.desc("Disconnect Eris from Discord.")
                            .cmd(cmd::admin::quit)
                            .owners_only(true)
                    })
            })
            .group("GURPS", |g| {
                g.command("st", |c| {
                    c.desc("Calculate Basic Lift and damage for a given ST.")
                        .cmd(cmd::gurps::calc_st)
                        .num_args(1)
                })
            })
            .group("Toys", |g| {
                g.command("fnord", |c| {
                    c.desc("Receive a message from the conspiracy.")
                        .cmd(cmd::misc::fnord)
                }).command("ddate", |c| {
                    c.desc("PERPETUAL DATE CONVERTER FROM GREGORIAN TO POEE CALENDAR")
                        .cmd(cmd::misc::ddate)
                })
            })
            .group("Randomizers", |g| {
                g.command("choose", |c| {
                    c.batch_known_as(&["decide", "pick"])
                        .desc("Choose between multiple comma-delimited options.")
                        .example("Option A, Option B, or Option C")
                        .cmd(cmd::random::choose)
                }).command("ask", |c| {
                        c.batch_known_as(&["eight", "8ball"])
                            .desc("Ask the Magic 8 Ball a yes-or-no question.")
                            .cmd(cmd::random::eight)
                    })
                    .command("flip", |c| c.desc("Flip a coin.").cmd(cmd::random::flip))
                    .command("roll", |c| {
                        c.desc("Calculate an expression in algebraic dice notation.")
                            .cmd(cmd::roll::dice)
                    })
            })
            .group("Tools", |g| {
                g.command("calc", |c| {
                    c
                .desc("A unit-aware precision calculator based on GNU units.")
                .cmd(cmd::util::calc)
                .min_args(1)
                .usage("expr[, into-unit]`\nFor details, see https://www.gnu.org/software/units/manual/units.html `\u{200B}")
                }).command("logs", |c| {
                    c.desc("Generate a log file for this channel to the current timestamp.")
                        .cmd(cmd::util::get_history)
                        .known_as("log")
                })
            }),
    );

    key::init(&mut client)?;
    client.start().map_err(SyncFailure::new)?;

    Ok(())
}
