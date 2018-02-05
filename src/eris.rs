use serenity::{self, prelude::*, model::prelude::*};

pub struct Handler;
impl EventHandler for Handler {
    fn message(&self, _: Context, message: Message) {
        match message.channel_id.get() {
            Ok(Channel::Guild(channel)) => {
                println!("{}--[{} #{}] {} <{}> {}",
                    channel.read().id,
                    channel.read().guild_id.get().unwrap().name,
                    channel.read().name(),
                    message.timestamp,
                    message.author.name,
                    message.content
                );
            }
            Ok(Channel::Group(channel)) => {
                println!("[{}] {} <{}> {}",
                    channel.read().name(),
                    message.timestamp,
                    message.author.name,
                    message.content
                );
            }
            Ok(Channel::Private(channel)) => {
                println!("[{}] {} <{}> {}",
                    channel.read().name(),
                    message.timestamp,
                    message.author.name,
                    message.content
                );
            }
            Ok(Channel::Category(channel)) => {
                println!("[{}] {} <{}> {}",
                    channel.read().name(),
                    message.timestamp,
                    message.author.name,
                    message.content
                );
            }
            Err(_) => {
                println!("[Unknown Channel] {} <{}> {}",
                    message.timestamp,
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
        ctx.set_game_name("to the crowd.");
    }

    fn reaction_add(&self, ctx: Context, re: Reaction) {
        use commands::random::roll_and_send;
        use data::DiceMessages;

        // Don't respond to our own reactions.
        if serenity::utils::with_cache(|cache| cache.user.id == re.user_id) { return; }

        // Reaction matcher.
        match re.emoji {
            // Reroll dice.
            ReactionType::Unicode(ref x) if x == "🎲" => {
                let mut data = ctx.data.lock();

                let mut map = if let Some(map) = data.get_mut::<DiceMessages>() {
                    map
                } else {
                    warn!("DiceMessages map is not initialised.");
                    return;
                };

                let dice = if let Some(dice) = map.get(&re.message_id) {
                    dice.clone()
                } else {
                    info!("Message is not in DiceMessages map.");
                    return;
                };

                if let Err(e) = roll_and_send(map, re.channel_id, re.user_id, dice) {
                    error!("Failed to repeat die roll: {}", e);
                } else {
                    match re.channel_id.message(re.message_id) {
                        Ok(m) => {
                            if let Err(e) = m.delete_reactions() {
                                warn!("Failed to clear reactions: {}", e);
                            }
                        }
                        Err(e) => warn!("Failed to get original message: {}", e),
                    }
                }
            }
            // An unconfigured reaction type.
            r => debug!("Unknown ReactionType: {:?}", r),
        }
    }
}

use failure::Error;
pub fn run() -> Result<(), Error> {
    use serenity::framework::standard::{StandardFramework, help_commands};
    use super::{data,commands};

    let mut client = Client::new(&::std::env::var("DISCORD_TOKEN")?, Handler)?;

    let info = serenity::http::get_current_application_info()?;

    data::init(&mut client);

    client.with_framework(StandardFramework::new()
        .configure(|c| { c
            .allow_dm(true)
            .allow_whitespace(false)
            // .blocked_guilds(hashset!{GuildId(1), GuildId(2)})
            // .blocked_users(hashset!{UserId(1), UserId(2)})
            .depth(5)
            // .disabled_commands(hashset!{"foo", "fnord"})
            .dynamic_prefix(::utils::select_prefix)
            .ignore_bots(true)
            .ignore_webhooks(true)
            .on_mention(true)
            .owners(hashset!{info.owner.id})
            .prefix(".")
            //.prefixes(hashset!{".", "!", "/"})
            .delimiters(hashset!{", or ", ", ", ",", " or ", " "})
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

            .group("Meta", |g| { g
                .command("logs", |c| { c
                    .desc("Generate a log file for this channel to the current timestamp.")
                    .cmd(commands::meta::get_history)
                    .known_as("log")
                })
                .command("playing", |c| { c
                    .desc("Set the currently displayed game tag.")
                    .cmd(commands::meta::set_playing)
                    .known_as("play")
                    .min_args(1)
                    .owners_only(true)
                })
                .command("prefix", |c| { c
                    .desc("Change the bot's command prefix on the current guild.")
                    .cmd(commands::meta::change_guild_prefix)
                    .guild_only(true)
                    .min_args(1)
                    .required_permissions(Permissions::ADMINISTRATOR)
                })
                .command("quit", |c| { c
                    .desc("Disconnect the bot from Discord.")
                    .cmd(commands::meta::quit)
                    .owners_only(true)
                })
                .command("nick", |c| { c
                    .desc("Change the bot's nickname on the current guild.")
                    .cmd(commands::meta::change_nick)
                    .guild_only(true)
                    .required_permissions(Permissions::ADMINISTRATOR)
                })
            })

            .group("Noisemakers", |g| { g
                .bucket("noise")
                .command("fnord", |c| { c
                    .desc("Transmits a message from the conspiracy.")
                    .cmd(commands::toys::fnord)
                })
                .command("trade", |c| { c
                    .desc("Sends out a kitten trading caravan.")
                    .cmd(commands::toys::trade)
                })
                .command("ddate", |c| { c
                    .desc("PERPETUAL DATE CONVERTER FROM GREGORIAN TO POEE CALENDAR")
                    .cmd(commands::toys::ddate)
                })
            })

            .group("Randomizers", |g| { g
                .command("choose", |c| { c
                    .batch_known_as(vec!["decide", "pick"])
                    .desc("Chooses between multiple options.")
                    .example("Option A, Option B, or Option C")
                    .cmd(commands::random::choose)
                    .min_args(2)
                })
                .command("ask", |c| { c
                    .batch_known_as(vec!["eight", "8ball"])
                    .desc("Ask the Magic 8 Ball a yes-or-no question.")
                    .cmd(commands::random::eight)
                })
                .command("flip", |c| { c
                    .desc("Flip a coin.")
                    .cmd(commands::random::flip)
                })
                .command("roll", |c| { c
                    .desc("Roll virtual dice in (limited) dice notation.")
                    .cmd(commands::random::roll)
                    .usage(indoc!("NxXdY±Z`, where…
                            *`X`* is the number of dice to be rolled.
                            *`Y`* is the number of sides on each die.
                            *`N`* is the number of times to repeat the entire roll (optional).
                            *`±`* and *`Z`* are an optional mathematical modifier and its argument.
                                This can be one of *`+`*, *`-`*, *`x`*, or *`/`*; or *`b`* or *`w`* to take the `b`est or `w`orst *`Z`* rolls.
                        
                        For game-specific skill checks, `roll <dice> vs T` compares the result to the target number *`T`*. This is currently implemented for GURPS, generic d20 (roll over), and generic d100 (roll under).
                        
                        For minimal usage, `roll` alone rolls 3d6.
                        
                        See also: https://en.wikipedia.org/wiki/Dice_notation `\u{200B}"
                    ))
                })
            })

            .group("GURPS", |g| { g
                .command("st", |c| { c
                    .desc("Calculate Basic Lift and Damage for a given ST.")
                    .cmd(commands::gurps::calc_st)
                    .num_args(1)
                })
                .command("reaction", |c| { c
                    .desc("Makes a reaction roll with a given modifier.")
                    .cmd(commands::gurps::reaction)
                    .known_as("react")
                    .num_args(1)
                    .usage("<modifier>")
                })
            })

            .group("Tools", |g| { g
                .command("calc", |c| { c
                    .desc("A unit-aware precision calculator using Rink.")
                    .cmd(commands::calc::calc)
                    .min_args(1)
                    .usage("expr`\nFor details, see https://github.com/tiffany352/rink-rs/wiki/Rink-Manual `\u{200B}")
                })
            }),
    );

    client.start()?;

    Ok(())
}
