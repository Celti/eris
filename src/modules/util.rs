// use chrono::{Date, Offset, Utc};
use crate::model::{/*MessageExt,*/ Owner};
use humantime::format_duration;
use serenity::model::id::{/*ChannelId,*/ MessageId};
//use serenity::model::misc::Mentionable;
use std::time::Duration;
//use std::io::{Seek, SeekFrom, Write};
use std::process::Command;
use sysinfo::{
    ProcessExt,
    SystemExt,
    System,
    get_current_pid
};

cmd!(About(ctx, msg, _args)
     aliases: ["about"],
     desc: "About the bot.",
     num_args: 0,
{
    let data = ctx.data.lock();
    let owner = data.get::<Owner>().expect("owner").to_user()?;
    let (guild_count, shard_count, thumbnail) = serenity::utils::with_cache(|cache| {
        (cache.guilds.len(), cache.shard_count, cache.user.face())
    });

    let sys = System::new();
    if let Some(process) = sys.get_process(get_current_pid()) {
        msg.channel_id.send_message(|m| m
            .embed(|e| e
                .description("I am Eris, Goddess of Discord (a dicebot historically, now an idiosyncratic entity written in [Rust](http://www.rust-lang.org/) using [Serenity](https://github.com/serenity-rs/serenity)).")
                .field("Admin", format!("Name: {}\nID: {}", owner.tag(), owner.id), true)
                .field("Links", "[Invite](https://discordapp.com/api/oauth2/authorize?client_id=256287298155577344&permissions=0&scope=bot)\n[Source](https://github.com/Celti/eris)", true)
                .field("Counts", format!("Servers: {}\nShards: {}", guild_count, shard_count), false)
                .field("System Info", format!("OS: {} {}\nUptime: {}",
                    sys_info::os_type().unwrap_or(String::from("OS Not Found")),
                    sys_info::os_release().unwrap_or(String::from("Release Not Found")),
                    format_duration(Duration::from_secs(sys.get_uptime()))), true)
                .field("Process Info", format!("Memory Usage: {} mB\nCPU Usage {}%\nUptime: {}",
                    process.memory()/1000, // convert to mB
                    (process.cpu_usage()*100.0).round()/100.0, // round to 2 decimals
                    format_duration(Duration::from_secs(sys.get_uptime() - process.start_time()))), true)
                .thumbnail(thumbnail)
                .colour(15_385_601)
        ))?;
    } else {
        msg.channel_id.send_message(|m| m
            .embed(|e| e
                .description("I am Eris, Goddess of Discord (a dicebot historically, now an idiosyncratic entity written in [Rust](http://www.rust-lang.org/) using [Serenity](https://github.com/serenity-rs/serenity)).")
                .field("Admin", format!("Name: {}\nID: {}", owner.tag(), owner.id), true)
                .field("Links", "[Invite](https://discordapp.com/api/oauth2/authorize?client_id=256287298155577344&permissions=0&scope=bot)\n[Source](https://github.com/Celti/eris)", true)
                .field("Counts", format!("Servers: {}\nShards: {}", guild_count, shard_count), false)
                .thumbnail(thumbnail)
                .colour(15_385_601)
        ))?;
    }
});

cmd!(Calc(_ctx, msg, args)
     aliases: ["calc"],
     desc: "A unit-aware precision calculator based on GNU units.",
     min_args: 1,
     usage: "expr[, into-unit]`\nFor details, see https://www.gnu.org/software/units/manual/units.html `\u{200B}",
{
    let expr = args.full().split(',');

    let output = Command::new("/usr/bin/units")
        .env("UNITS_ENGLISH", "US")
        .arg("--terse")
        .arg("--")
        .args(expr)
        .output()?;

    msg.reply(&String::from_utf8_lossy(&output.stdout))?;
});

// cmd!(LogFile(_ctx, msg, args)
//      aliases: ["log", "logs"],
//      desc: "Generate a log file for a channel.",
//      max_args: 3,
//      usage: "[channel [from-msg-id [to-msg-id]]]`\nDefaults to the entirety of the current channel. `\u{200B}",
// {
//     let channel_id = match args.single::<String>() {
//         Err(_) => msg.channel_id,
//         Ok(s)  => {
//             match serenity::utils::parse_channel(&s) {
//                 Some(id) => ChannelId(id),
//                 None     => s.parse::<u64>().map(ChannelId)?,
//             }
//         }
//     };

//     let from_id = match args.single::<String>() {
//         Ok(s)  => s.trim().parse::<u64>().map(|i| MessageId(i - 1))?,
//         Err(_) => MessageId(0),
//     };

//     let until_id = match args.single::<String>() {
//         Ok(s)  => s.trim().parse::<u64>().map(MessageId)?,
//         Err(_) => MessageId(std::u64::MAX),
//     };

//     let mut buf = tempfile::tempfile()?;
//     let mut next_id = from_id;
//     let mut next_ts = Date::from_utc(from_id.created_at().date(), Utc.fix());

//     'outer: loop {
//         let batch   = channel_id.messages(|m| m.after(next_id).limit(100))?;
//         let count   = batch.len();
//         let last_id = batch[0].id;

//         for message in batch.into_iter().rev() {
//             if message.timestamp.date() > next_ts {
//                 next_ts = message.timestamp.date();
//                 writeln!(&mut buf, "-- Day changed {}", next_ts.format("%A, %e %B %Y %Z"));
//             }

//             writeln!(&mut buf, "[{}] {}",
//                      message.timestamp.format("%H:%M:%S"),
//                      message.to_logstr())?;

//             if message.id == until_id {
//                 break 'outer;
//             }
//         }

//         if count < 100 || next_id == last_id {
//             break;
//         }

//         next_id = last_id;
//     }

//     buf.seek(SeekFrom::Start(0))?;

//     let channel  = channel_id.name().unwrap_or_else(|| channel_id.to_string());
//     let filename = format!("{}-{}.txt", channel, msg.timestamp);

//     let content = format!("Logs for {} from {} to {}.",
//                           channel_id.mention(),
//                           from_id.created_at(),
//                           until_id.created_at());

//     msg.channel_id.send_files(Some((&buf, &*filename)), |m| m.reactions(Some('❌')).content(content))?;
// });

cmd!(TimeStamp(_ctx, msg, args)
     aliases: ["when", "time", "timestamp", "date", "datestamp"],
     desc: "Get the timestamp of the specified Discord snowflake (object ID).",
{
    // All snowflakes are the same for timestamps. MessageId has the desired method.
    let id = MessageId({
        let s = args.single::<String>()?;

        if s.starts_with("<@&") {
            serenity::utils::parse_role(&s)
        } else if s.starts_with("<@") {
            serenity::utils::parse_username(&s)
        } else if s.starts_with("<#") {
            serenity::utils::parse_channel(&s)
        } else {
            str::parse::<u64>(&s).ok()
        }
    }.ok_or("Could not parse snowflake!")?);

    reply!(msg, "Snowflake {} was created at {} UTC.", id, id.created_at());
});

grp![About, Calc, TimeStamp];
