#![recursion_limit = "1024"]

#[macro_use] extern crate failure_derive;
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
        std::process::exit(1);
    }
}

fn run() -> Result<(), Error> {
    // TODO write our own help command builder.
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

            .group("Meta", |g| {
                g.command("help", |c| {
                    c.desc("Displays help for available commands.")
                        .exec_help(help_commands::plain)
                        .known_as("halp")
                }).command("playing", |c| {
                        c.desc("Set the currently displayed game tag.")
                            .exec(commands::meta::set_playing)
                            .known_as("play")
                            .owners_only(true)
                    })
                    .command("quit", |c| {
                        c.desc("Disconnect the bot from Discord.")
                            .exec(commands::meta::quit)
                            .owners_only(true)
                    })
                    .command("nick", |c| {
                        c.desc("Change the bot's nickname on the current guild.")
                            .exec(commands::meta::nick)
                            .guild_only(true)
                            .owners_only(true)
                    })
            })

            .group("Noisemakers", |g| {
                g.bucket("noise")
                    .command("fnord", |c| {
                        c.desc("Transmits a message from the conspiracy.")
                            .exec(commands::toys::fnord)
                    })
                    .command("trade", |c| {
                        c.desc("Sends out a kitten trading caravan.")
                            .exec(commands::toys::trade)
                    })
                    .command("ddate", |c| {
                        c.desc("PERPETUAL DATE CONVERTER FROM GREGORIAN TO POEE CALENDAR")
                            .exec(commands::toys::ddate)
                    })
            })

            .group("Randomizers", |g| {
                g.command("choose", |c| {
                    c.desc("Chooses between multiple options.")
                        .batch_known_as(vec!["decide", "pick"])
                        .example("Option A, Option B, or Option C")
                        .exec(commands::random::choose)
                }).command("ask", |c| {
                        c.desc("Ask the Magic 8 Ball a yes-or-no question.")
                            .batch_known_as(vec!["eight", "8ball"])
                            .exec(commands::random::eight)
                    })
                    .command("flip", |c| {
                        c.desc("Flip a coin.")
                            .exec(commands::random::flip)
                    })
                    .command("roll", |c| {
                        c.desc("Roll virtual dice in dice algebra notation.")
                            .exec(commands::random::roll)
                            .usage(
                                "`**Simplest:** `!roll` rolls 3d6. \
                        **Minimal:** `!roll AdX`, where… \
                            **A** is the number of dice to be rolled. \
                            **X** is the number of sides on each die. \
                        **Maximal:** `!roll NxAdX@M`, which adds… \
                            **N** repeats this roll *N* times and displays separate results. \
                            **@** adds a mathematical modifier: *+*, *-*, *x*, or */*. \
                            **M** is the the argument to the modifier. \
                        **Game-Specific:** `!roll AdXbM`, `!roll AdXwM`, `!roll 3d6 vs T` \
                            *b* and *w* indicate to take the best or worst **M** rolls (Generic). \
                            *vs* will compare the roll to the target number **T** (GURPS). \
                        \
                        See also: https://en.wikipedia.org/wiki/Dice_notation`",
                            )
                    })
            })

            .group("GURPS", |g| {
                g.command("st", |c| {
                    c.desc("Calculate Basic Lift and Damage for a given ST.")
                        .exec(commands::gurps::st)
                })
            })

            .group("Tools", |g| {
                g.command("calc", |c| {
                    c.desc(
                        "A unit-aware precision calculator using Rink.\
                         See also: https://github.com/tiffany352/rink-rs/wiki/Rink-Manual",
                    ).exec(commands::calc::calc)
                })
            }),
    );

    Ok(client.start()?)
}
