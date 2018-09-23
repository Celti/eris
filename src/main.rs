#![feature(nll)]
#![feature(tool_lints)]
#![feature(try_blocks)]

#![allow(proc_macro_derive_resolution_fallback)]    // FIXME diesel 1.4
#[macro_use] extern crate diesel;                   // FIXME diesel 1.4

mod cmd;
mod schema;
mod types;
mod util;

use crate::types::*;
use exitfailure::ExitFailure;
use failure::SyncFailure;
use maplit::hashset;
use serenity::prelude::*;

struct Eris;
impl EventHandler for Eris {
    fn message(&self, mut context: Context, message: Message) {
        util::last_seen_id(&mut context, &message);
        util::log_message(&message);
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(s) = ready.shard {
            log::info!("Logged in as '{}' on {}/{}", ready.user.name, s[0], s[1]);
        } else {
            log::info!("Logged in as '{}'", ready.user.name);
        }

        // TODO get name from persistent store
        ctx.set_game("with fire.");
    }

    fn reaction_add(&self, ctx: Context, re: Reaction) {
        use crate::types::DiceCache;
        use serenity::framework::standard::{Args, CommandError};

        let bot_id = serenity::utils::with_cache(|cache| cache.user.id);

        // Don't respond to our own reactions.
        if bot_id == re.user_id {
            return;
        }

        // Reaction matcher.
        match re.emoji {
            // Reroll dice.
            ReactionType::Unicode(ref x) if x == "🎲" => {
                let mut map = ctx.data.lock();
                let cache = map.get_mut::<DiceCache>().unwrap();

                let result: Result<(), CommandError> = try {
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
                        log::info!("Die roll is not in message cache.");
                    };
                };

                if let Err(err) = result {
                   log::error!("error repeating dice roll: {:?}", err);
                }
            }

            // Delete message.
            ReactionType::Unicode(ref x) if x == "❌" => {
                let result: Result<(), CommandError> = try {
                    let msg = re.message()?;

                    if msg.author.id == bot_id {
                        msg.delete()?;
                    };
                };

                if let Err(err) = result {
                   log::error!("error deleting message: {:?}", err);
                }
            }

            // An unconfigured reaction type.
            r => log::debug!("Unknown ReactionType: {:?}", r),
        }
    }
}

fn main() -> Result<(), ExitFailure> {
    dotenv::dotenv().ok();
    log_panics::init();
    pretty_env_logger::init();

    use serenity::framework::standard::{help_commands, DispatchError, StandardFramework};

    let token = std::env::var("DISCORD_TOKEN")?;
    let mut client = Client::new(&token, Eris).map_err(SyncFailure::new)?;
    let info = serenity::http::get_current_application_info().map_err(SyncFailure::new)?;

    client.with_framework(StandardFramework::new()
        .configure(|c| { c
            .allow_dm(true)
            .allow_whitespace(false)
            // .blocked_guilds(hashset!{GuildId(1), GuildId(2)})
            // .blocked_users(hashset!{UserId(1), UserId(2)})
            .case_insensitivity(true)
            // .delimiters(&[", or ", ", ", ",", " or ", " "])
            // .depth(5)
            // .disabled_commands(hashset!{"foo", "fnord"})
            .dynamic_prefix(util::dynamic_prefix)
            .ignore_bots(true)
            .ignore_webhooks(true)
            .no_dm_prefix(true)
            .on_mention(true)
            .owners(hashset!{info.owner.id})
        })

        .after(|_ctx, _msg, cmd, res| match res {
            Ok(()) => log::info!("Successfully processed command '{}'", cmd),
            Err(e) => log::error!("Error processing command '{}': {:?}", cmd, e),
        })

        .on_dispatch_error(|_ctx, msg, err| match err {
            DispatchError::OnlyForDM => {
                msg.reply("This command is only available in DMs.").ok();
            }

            DispatchError::OnlyForGuilds => {
                msg.reply("This command is only available in servers.").ok();
            }

            DispatchError::RateLimited(t) => {
                msg.reply(&format!("Ratelimited; please wait at least {} seconds.", t)).ok();
            }

            DispatchError::NotEnoughArguments { min: m, given: n } => {
                msg.reply(&format!("This command takes at least {} arguments (gave {}).", m, n)).ok();
            }

            DispatchError::TooManyArguments { max: m, given: n } => {
                msg.reply(&format!("This command takes at most {} arguments (gave {}).", m, n)).ok();
            }

            _ => {
                log::info!("Command not executed: {:?}", err);
            }
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
            .command("linear", |c| { c
                .cmd(cmd::gurps::calc_linear)
                .desc("Calculate the linear value for a given modifier (reverse SSR).")
                .known_as("super")
                .num_args(1)
            })
            .command("range", |c| { c
                .cmd(cmd::gurps::calc_range)
                .desc("Calculate range penalty for a given distance.")
                .min_args(1)
            })
            .command("sm", |c| { c
                .cmd(cmd::gurps::calc_sm)
                .desc("Calculate size modifier for a given measurement.")
                .min_args(1)
            })
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
                .desc("Generate a log file for the specified channel. Defaults to the entirety of the current channel.")
                .max_args(3)
                .known_as("log")
                .usage("[channel [from_id [to_id]]]")
            })
            .command("when", |c| { c
                .cmd(cmd::util::get_timestamp)
                .desc("Get the timestamp of the specified Discord snowflake (object ID).")
                .batch_known_as(&["time", "timestamp", "date", "datestamp"])
            })
        })
        .group("Toys", |g| { g
            .command("fnord", |c| { c
                .desc("Receive a message from the conspiracy.")
                .cmd(cmd::misc::fnord)
            })
            .command("ddate", |c| { c
                .cmd(cmd::misc::get_ddate)
                .desc("PERPETUAL DATE CONVERTER FROM GREGORIAN TO POEE CALENDAR")
            })
        })
    );

    db_init(&mut client)?;
    client.start().map_err(SyncFailure::new)?;

    Ok(())
}
