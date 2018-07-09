#![feature(rust_2018_preview, use_extern_macros, nll)]

mod util;
mod cmd;

use crate::util::cached_display_name;
use log::{log, error, warn, info}; // debug, trace
use serenity::{prelude::*, model::prelude::*};
use std::error::Error;

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
}

fn main() -> Result<(), Box<Error>> {
    dotenv::dotenv().ok();
    log_panics::init();
    env_logger::init();

    use serenity::framework::standard::{StandardFramework, help_commands};

    let token = std::env::var("DISCORD_TOKEN")?;
    let mut client = Client::new(&token, Eris)?;

    client.with_framework(StandardFramework::new()
        .configure(|c| { c
            .allow_dm(true)
            .allow_whitespace(false)
            // .blocked_guilds(hashset!{GuildId(1), GuildId(2)})
            // .blocked_users(hashset!{UserId(1), UserId(2)})
            // .depth(5)
            // .disabled_commands(hashset!{"foo", "fnord"})
            // .dynamic_prefix(crate::utils::select_prefix)
            .ignore_bots(true)
            .ignore_webhooks(true)
            .on_mention(true)
            .prefix("$$")
            // .owners(hashset!{info.owner.id})
            // .delimiters(&[", or ", ", ", ",", " or ", " "])
            .case_insensitivity(true)
            })

        .after(|_ctx, _msg, cmd, res| {
            match res {
                Ok(()) => info!("Successfully processed command '{}'", cmd),
                Err(e) => error!("Error processing command '{}': {:?}", cmd, e),
            }
        })

        .on_dispatch_error(|_ctx, msg, err| {
            // TODO match on DispatchError enum and customise responses.
            let _ = msg.channel_id.say(&format!("Could not execute command: {:?}", err));
        })

        .help(help_commands::with_embeds)

        .group("Admin", |g| { g
            .command("playing", |c| { c
                .desc("Set the currently displayed game tag.")
                .cmd(cmd::admin::set_playing)
                .known_as("play")
                .min_args(1)
                .owners_only(true)
            })
            .command("nick", |c| { c
                .desc("Change Eris's nickname on the current guild.")
                .cmd(cmd::admin::change_nick)
                .guild_only(true)
                .required_permissions(Permissions::ADMINISTRATOR)
            })
        })

        .group("Toys", |g| { g
            .command("fnord", |c| { c
                .desc("Receive a message from the conspiracy.")
                .cmd(cmd::misc::fnord)
            })
            .command("ddate", |c| { c
                .desc("PERPETUAL DATE CONVERTER FROM GREGORIAN TO POEE CALENDAR")
                .cmd(cmd::misc::ddate)
            })
        })

        .group("Randomizers", |g| { g
            .command("choose", |c| { c
                .batch_known_as(&["decide", "pick"])
                .desc("Choose between multiple comma-delimited options.")
                .example("Option A, Option B, or Option C")
                .cmd(cmd::random::choose)
            })
            .command("ask", |c| { c
                .batch_known_as(&["eight", "8ball"])
                .desc("Ask the Magic 8 Ball a yes-or-no question.")
                .cmd(cmd::random::eight)
            })
            .command("flip", |c| { c
                .desc("Flip a coin.")
                .cmd(cmd::random::flip)
            })
            .command("roll", |c| { c
                .desc("Calculate an expression in algebraic dice notation.")
                .cmd(cmd::roll::roll)
            })
        })

        .group("Tools", |g| { g
            .command("calc", |c| { c
                .desc("A unit-aware precision calculator based on GNU units.")
                .cmd(cmd::util::calc)
                .min_args(1)
                .usage("expr[, into-unit]`\nFor details, see https://www.gnu.org/software/units/manual/units.html `\u{200B}")
            })
            .command("logs", |c| { c
                .desc("Generate a log file for this channel to the current timestamp.")
                .cmd(cmd::util::get_history)
                .known_as("log")
            })
        })
    );

    Ok(client.start()?)
}
