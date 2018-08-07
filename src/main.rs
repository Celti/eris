#![allow(proc_macro_derive_resolution_fallback)] // diesel <= 1.3.2
#![feature(rust_2018_preview)]
#![feature(entry_or_default)]
#![feature(nll)]
//#![feature(use_extern_macros)]

// FIXME use_extern_macros
#[macro_use] extern crate diesel;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate maplit;
#[macro_use] extern crate serenity;

mod cmd;
mod db;
mod types;
mod util;
mod schema;

use crate::util::cached_display_name;
use exitfailure::ExitFailure;
use failure::SyncFailure;
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
        use crate::types::DiceCache;
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

                #[cfg_attr(feature = "cargo-clippy", allow(unit_arg))]
                let result: Result<(), CommandError> = do catch {
                    if let Some(expr) = cache.remove(&re.message_id) {
                        re.message()?.delete_reactions()?;

                        let args = Args::new(&expr, &[" ".to_string()]);
                        let name = re.user_id.mention();
                        let roll = cmd::roll::process_args(args)?;
                        let sent = re.channel_id.send_message(|m| { m
                            .content(format!("**{} rolled:**{}", name, roll))
                            .reactions(vec!['🎲'])
                        })?;

                        cache.insert(sent.id, expr);
                    } else {
                        return info!("Die roll is not in message cache.");
                    }
                };

                if let Err(err) = result {
                   return error!("error repeating dice roll: {:?}", err);
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
    let info = serenity::http::get_current_application_info().map_err(SyncFailure::new)?;

    client.with_framework(StandardFramework::new()
        .configure(|c| { c
            .allow_dm(true)
            .allow_whitespace(false)
            // .blocked_guilds(hashset!{GuildId(1), GuildId(2)})
            // .blocked_users(hashset!{UserId(1), UserId(2)})
            // .depth(5)
            // .disabled_commands(hashset!{"foo", "fnord"})
            .dynamic_prefix(db::dynamic_prefix)
            .ignore_bots(true)
            .ignore_webhooks(true)
            .on_mention(true)
            .prefix("$$")
            .owners(hashset!{info.owner.id})
            // .delimiters(&[", or ", ", ", ",", " or ", " "])
            .case_insensitivity(true)
        })

        .after(|_ctx, _msg, cmd, res| match res {
            Ok(()) => info!("Successfully processed command '{}'", cmd),
            Err(e) => error!("Error processing command '{}': {:?}", cmd, e),
        })

        .on_dispatch_error(|_ctx, msg, err| {
            // TODO match on DispatchError enum and customise responses.
            let _ = msg.channel_id.say(&format!("Could not execute command: {:?}", err));
        })

        .help(help_commands::with_embeds)

        .unrecognised_command(util::bareword_handler)

        .group("Admin", |g| { g
            .command("nick", |c| { c
                .cmd(cmd::admin::change_nick)
                .desc("Change Eris's nickname on the current guild.")
                .guild_only(true)
                .required_permissions(Permissions::ADMINISTRATOR)
            })
            .command("playing", |c| { c
                .cmd(cmd::admin::set_playing)
                .desc("Set the currently displayed game tag.")
                .known_as("play")
                .min_args(1)
                .owners_only(true)
            })
            .command("prefix", |c| { c
                .cmd(cmd::admin::change_prefix)
                .desc("Change Eris's command prefix on the current guild or channel.")
                .required_permissions(Permissions::ADMINISTRATOR)
            })
            .command("topic", |c| { c
                .cmd(cmd::admin::change_topic)
                .desc("Change the current channel's topic.")
                .guild_only(true)
                .required_permissions(Permissions::MANAGE_CHANNELS)
            })
            .command("quit", |c| { c
                .cmd(cmd::admin::quit)
                .desc("Disconnect Eris from Discord.")
                .owners_only(true)
            })
        })
        .group("GURPS", |g| { g
            .command("st", |c| { c
                .cmd(cmd::gurps::calc_st)
                .desc("Calculate Basic Lift and damage for a given ST.")
                .num_args(1)
            })
        })
        .group("Memory", |g| { g
            .command("recall", |c| { c
                .cmd(cmd::memory::recall)
                .desc("Retrieve a keyword definition.")
                .num_args(1)
            })
            .command("next", |c| { c
                .cmd(cmd::memory::next)
                .desc("Retrieve the next keyword definition.")
                .num_args(0)
            })
            .command("prev", |c| { c
                .cmd(cmd::memory::prev)
                .desc("Retrieve the previous keyword definition.")
                .num_args(0)
            })
            .command("details", |c| { c
                .cmd(cmd::memory::details)
                .desc("Retrieve metadata for the current keyword definition.")
                .num_args(0)
            })
            .command("remember", |c| { c
                .cmd(cmd::memory::remember)
                .desc("Add a new keyword definition.")
                .min_args(2)
            })
            .command("embed", |c| { c
                .cmd(cmd::memory::remember_embed)
                .desc("Add a new keyword embed.")
                .min_args(2)
            })
            .command("forget", |c| { c
                .cmd(cmd::memory::forget)
                .desc("Forget a specific keyword definition.")
                .min_args(2)
            })
            .command("set", |c| { c
                .cmd(cmd::memory::set)
                .desc("Set keyword options.")
                .num_args(3)
            })
        })
        .group("Randomizers", |g| { g
            .command("choose", |c| { c
                .cmd(cmd::random::choose)
                .desc("Choose between multiple comma-delimited options.")
                .batch_known_as(&["decide", "pick"])
                .example("Option A, Option B, or Option C")
            })
            .command("ask", |c| { c
                .cmd(cmd::random::eight)
                .desc("Ask the Magic 8 Ball a yes-or-no question.")
                .batch_known_as(&["eight", "8ball"])
            })
            .command("flip", |c| { c
                .cmd(cmd::random::flip)
                .desc("Flip a coin.")
            })
            .command("roll", |c| { c
                .cmd(cmd::roll::dice)
                .desc("Calculate an expression in algebraic dice notation.")
            })
        })
        .group("Tools", |g| { g
            .command("calc", |c| { c
                .cmd(cmd::util::calc)
                .desc("A unit-aware precision calculator based on GNU units.")
                .min_args(1)
                .usage("expr[, into-unit]`\nFor details, see https://www.gnu.org/software/units/manual/units.html `\u{200B}")
            })
            .command("logs", |c| { c
                .cmd(cmd::util::get_history)
                .desc("Generate a log file for this channel to the current timestamp.")
                .known_as("log")
            })
            .command("when", |c| { c
                .cmd(cmd::util::get_timestamp)
                .desc("Get the timestamp of the specified Discord snowflake (message ID).")
                .batch_known_as(&["time", "timestamp", "date", "datestamp"])
            })
        })
        .group("Toys", |g| { g
            .command("fnord", |c| { c
                .desc("Receive a message from the conspiracy.")
                .cmd(cmd::misc::fnord)
            })
            .command("ddate", |c| { c
                .cmd(cmd::misc::ddate)
                .desc("PERPETUAL DATE CONVERTER FROM GREGORIAN TO POEE CALENDAR")
            })
        })
    );

    db::init(&mut client)?;
    client.start().map_err(SyncFailure::new)?;

    Ok(())
}
