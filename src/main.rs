#![recursion_limit = "1024"]
#![feature(match_default_bindings)]

#[macro_use] extern crate failure_derive;
#[macro_use] extern crate indoc;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate maplit;
#[macro_use] extern crate matches;
#[macro_use] extern crate serenity;

extern crate chrono;
extern crate ddate;
extern crate env_logger;
extern crate failure;
extern crate fnorder;
extern crate rand; // TODO use ring instead
extern crate regex;
extern crate rink;
extern crate typemap;

mod commands;
mod data;
mod eris;
mod ext;
mod utils;

use failure::Error;
use serenity::prelude::*;

fn main() {
    utils::init_env_logger();

    if let Err(ref err) = run() {
        error!("Application error: {}", err);

        let mut chain = err.cause().cause();
        while let Some(cause) = chain {
            error!("Caused by: {}", cause);
            chain = cause.cause();
        }

        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    use serenity::framework::standard::help_commands;

    let token = std::env::var("DISCORD_TOKEN")?;
    let mut client = Client::new(&token, eris::Handler);
    let info = serenity::http::get_current_application_info()?;

    client.with_framework(
        serenity::framework::StandardFramework::new()
            .configure(|c| {
                c.allow_dm(true)
                .allow_whitespace(false)
                // .blocked_guilds(hashset!{GuildId(1), GuildId(2)})
                // .blocked_users(hashset!{UserId(1), UserId(2)})
                .depth(5)
                // .disabled_commands(hashset!{"foo", "fnord"})
                //.dynamic_prefix(utils::select_prefix)
                .ignore_bots(true)
                .ignore_webhooks(true)
                .on_mention(true)
                .owners(hashset!{info.owner.id})
                .prefix(".")
                //.prefixes(vec![".", "!", "/"])
                .delimiters(vec![", or ", ", ", ",", " or ", " "])
                .case_insensitivity(true)
            })

            .after(|_ctx, _msg, _cmd, res| {
                if let Err(why) = res {
                    error!("Error sending message: {:?}", why);
                }
            })

            .group("Meta", |g| { g
                .command("help", |c| { c
                    .desc("Displays help for available commands.")
                    .exec_help(help_commands::with_embeds)
                    .known_as("halp")
                })
                .command("playing", |c| { c
                    .desc("Set the currently displayed game tag.")
                    .exec(commands::meta::set_playing)
                    .known_as("play")
                    .min_args(1)
                    .owners_only(true)
                })
                .command("quit", |c| { c
                    .desc("Disconnect the bot from Discord.")
                    .exec(commands::meta::quit)
                    .owners_only(true)
                })
                .command("nick", |c| { c
                    .desc("Change the bot's nickname on the current guild.")
                    .exec(commands::meta::nick)
                    .guild_only(true)
                    .min_args(1)
                    .owners_only(true)
                })
            })

            .group("Noisemakers", |g| { g
                .bucket("noise")
                .command("fnord", |c| { c
                    .desc("Transmits a message from the conspiracy.")
                    .exec(commands::toys::fnord)
                })
                .command("trade", |c| { c
                    .desc("Sends out a kitten trading caravan.")
                    .exec(commands::toys::trade)
                })
                .command("ddate", |c| { c
                    .desc("PERPETUAL DATE CONVERTER FROM GREGORIAN TO POEE CALENDAR")
                    .exec(commands::toys::ddate)
                })
            })

            .group("Randomizers", |g| { g
                .command("choose", |c| { c
                    .batch_known_as(vec!["decide", "pick"])
                    .desc("Chooses between multiple options.")
                    .example("Option A, Option B, or Option C")
                    .exec(commands::random::choose)
                    .min_args(2)
                })
                .command("ask", |c| { c
                    .batch_known_as(vec!["eight", "8ball"])
                    .desc("Ask the Magic 8 Ball a yes-or-no question.")
                    .exec(commands::random::eight)
                })
                .command("flip", |c| { c
                    .desc("Flip a coin.")
                    .exec(commands::random::flip)
                })
                .command("roll", |c| { c
                    .desc("Roll virtual dice in (limited) dice notation.")
                    .exec(commands::random::roll)
                    .min_args(1)
                    .usage(indoc!("NxXdY±Z`, where…
                            *`X`* is the number of dice to be rolled.
                            *`Y`* is the number of sides on each die.
                            *`N`* is the number of times to repeat the entire roll (optional).
                            *`±`* and *`Z`* are an optional mathematical modifier and its argument.
                                This can be one of *`+`*, *`-`*, *`x`*, or *`/`*; or *`b`* or *`w`* to take the `b`est or `w`orst *`Z`* rolls.
                        
                        For game-specific skill checks, `roll <dice> vs T` compares the result to the target number *`T`*. This is currently implemented for GURPS, generic d20 (roll over), and generic d100 (roll under).
                        
                        For minimal usage, `roll` alone rolls 3d6.
                        
                        See also: https://en.wikipedia.org/wiki/Dice_notation `​"
                    ))
                })
            })

            .group("GURPS", |g| { g
                .command("st", |c| { c
                    .desc("Calculate Basic Lift and Damage for a given ST.")
                    .exec(commands::gurps::st)
                    .num_args(1)
                })
            })

            .group("Tools", |g| { g
                .command("calc", |c| { c
                    .desc("A unit-aware precision calculator using Rink.")
                    .exec(commands::calc::calc)
                    .min_args(1)
                    .usage("expr`\nFor details, see https://github.com/tiffany352/rink-rs/wiki/Rink-Manual `​")
                })
            }),
    );

    client.start()?;

    Ok(())
}
