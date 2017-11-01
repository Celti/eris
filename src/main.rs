#![recursion_limit = "1024"]
#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate serenity;
extern crate chrono;
extern crate fnorder;
extern crate rand;
extern crate regex;
extern crate rink;

mod commands;
mod errors;
mod logger;

use errors::*;

fn main() {
    logger::init(log::LogLevel::Info).unwrap();

    if let Err(ref e) = run() {
        use error_chain::ChainedError;
        error!("{}", e.display_chain());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    use serenity::client::{Client, Context, EventHandler};
    use serenity::framework::StandardFramework;
    use serenity::framework::standard::help_commands;
    use serenity::model::{Ready, Game, OnlineStatus};
    use std::collections::HashSet;

    struct Handler;

    impl EventHandler for Handler {
        fn on_ready(&self, context: Context, ready: Ready) {
            let game = Some(Game::playing("you all like a fiddle."));
            let status = OnlineStatus::Idle;
            let afk = false;

            context.set_presence(game, status, afk);
            info!("Connected as {}", ready.user.name);
        }
    }

    let token = std::env::var("DISCORD_TOKEN").chain_err(
        || "could not get Discord authentication token",
    )?;

    let mut client = Client::new(&token, Handler);

    let info = serenity::http::get_current_application_info().chain_err(
        || "could not get Discord application info",
    )?;

    client.with_framework(
        StandardFramework::new()
            .configure(|c| { c
                .allow_dm(true)
                .allow_whitespace(false)
                // .blocked_guilds(hashset!{GuildId(1), GuildId(2)})
                // .blocked_users(hashset!{UserId(1), UserId(2)})
                .depth(5)
                // .disabled_commands(hashset!{"foo", "fnord"})
                // .dynamic_prefix(|_ctx, _msg| Some(String::from("!")))
                .ignore_bots(true)
                .ignore_webhooks(true)
                .on_mention(true)
                .owners(hashset!{info.owner.id})
                .prefixes(hashset!{".", "!", "/"})
                .delimiters(hashset!{", ", ",", ", or ", " or ", " "})
                .case_insensitivity(true)
            })
            .command("help", |c| c.exec_help(help_commands::with_embeds))
            .command("fnord", |c| c.exec_str(&fnorder::fnorder()))
            .command("choose", |c| {
                c.exec(commands::random::choose)
                    .known_as("decide")
                    .known_as("pick")
            })
            .command("8ball", |c| c.exec(commands::random::eight))
            .command("flip", |c| c.exec(commands::random::flip))
            .command("roll", |c| c.exec(commands::random::roll))
            .command("st", |c| c.exec(commands::gurps::st))
            .command("calc", |c| c.exec(commands::calc::calc)),
    );

    client.start().chain_err(|| "failed to start shard")
}
