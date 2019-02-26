// use chrono::{Date, Offset, Utc};
use crate::model::Owner;
use humantime::format_duration;
use serenity::model::id::MessageId;
//use serenity::model::misc::Mentionable;
use std::time::Duration;
//use std::io::{Seek, SeekFrom, Write};
use std::process::Command;
use sysinfo::{get_current_pid, ProcessExt, System, SystemExt};
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;

#[command]
#[description = "About this bot."]
#[num_args(0)]
fn about(ctx: &mut Context, msg: &Message) -> CommandResult {
    let data = ctx.data.read();
    let owner = data.get::<Owner>().expect("owner").to_user(&ctx)?;
    let (guild_count, shard_count, thumbnail) = serenity::utils::with_cache(&ctx.cache, |cache| {
        (cache.guilds.len(), cache.shard_count, cache.user.face())
    });

    let sys = System::new();
    if let Some(process) = sys.get_process(get_current_pid()) {
        msg.channel_id.send_message(&ctx.http, |m| m
            .embed(|e| e
                .description("I am Eris, Goddess of Discord (a dicebot historically, now an idiosyncratic entity written in [Rust](http://www.rust-lang.org/) using [Serenity](https://github.com/serenity-rs/serenity)).")
                .field("Admin", format!("Name: {}\nID: {}", owner.tag(), owner.id), true)
                .field("Links", "[Invite](https://discordapp.com/api/oauth2/authorize?client_id=256287298155577344&permissions=0&scope=bot)\n[Source](https://github.com/Celti/eris)", true)
                .field("Counts", format!("Servers: {}\nShards: {}", guild_count, shard_count), false)
                .field("System Info", format!("OS: {} {}\nUptime: {}",
                    sys_info::os_type().unwrap_or_else(|_| String::from("OS Not Found")),
                    sys_info::os_release().unwrap_or_else(|_| String::from("Release Not Found")),
                    format_duration(Duration::from_secs(sys.get_uptime()))), true)
                .field("Process Info", format!("Memory Usage: {} mB\nCPU Usage {}%\nUptime: {}",
                    process.memory()/1000, // convert to mB
                    (process.cpu_usage()*100.0).round()/100.0, // round to 2 decimals
                    format_duration(Duration::from_secs(sys.get_uptime() - process.start_time()))), true)
                .thumbnail(thumbnail)
                .colour(15_385_601)
        ))?;
    } else {
        msg.channel_id.send_message(&ctx.http, |m| m
            .embed(|e| e
                .description("I am Eris, Goddess of Discord (a dicebot historically, now an idiosyncratic entity written in [Rust](http://www.rust-lang.org/) using [Serenity](https://github.com/serenity-rs/serenity)).")
                .field("Admin", format!("Name: {}\nID: {}", owner.tag(), owner.id), true)
                .field("Links", "[Invite](https://discordapp.com/api/oauth2/authorize?client_id=256287298155577344&permissions=0&scope=bot)\n[Source](https://github.com/Celti/eris)", true)
                .field("Counts", format!("Servers: {}\nShards: {}", guild_count, shard_count), false)
                .thumbnail(thumbnail)
                .colour(15_385_601)
        ))?;
    }

    Ok(())
}

#[command]
#[description = "A unit-aware precision calculator based on GNU units."]
#[min_args(1)]
#[usage("expr[, into-unit]`\nFor details, see https://www.gnu.org/software/units/manual/units.html `\u{200B}")]
fn calc(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let expr = args.message().split(',');

    let output = Command::new("/usr/bin/units")
        .env("UNITS_ENGLISH", "US")
        .arg("--terse")
        .arg("--")
        .args(expr)
        .output()?;

    msg.reply(&ctx, &String::from_utf8_lossy(&output.stdout))?;

    Ok(())
}

#[command]
#[aliases("when", "time", "timestamp", "date", "datestamp")]
#[description("Get the timestamp of the specified Discord snowflake (object ID).")]
#[num_args(1)]
#[usage("expr[, into-unit]`\nFor details, see https://www.gnu.org/software/units/manual/units.html `\u{200B}")]
fn timestamp(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    // All snowflakes are the same for timestamps. MessageId has the desired method.
    let id = MessageId({
        let s = args.single::<String>()?;
        serenity::utils::parse_mention(&s)
            .or_else(|| str::parse::<u64>(&s).ok())
            .ok_or("Could not parse snowflake!")?
    });

    reply!(ctx, msg, "Snowflake {} was created at {} UTC.", id, id.created_at());

    Ok(())
}

group!({
    name: "util",
    options: {},
    commands: [about, calc, timestamp]
});
