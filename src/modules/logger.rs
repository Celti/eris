use chrono::{Date, Offset, Utc};
use crate::model::MessageExt;
use serenity::framework::standard::CommandError;
use serenity::model::id::{ChannelId, GuildId, MessageId};
use serenity::model::misc::Mentionable;
use std::io::{Seek, SeekFrom, Write};
use std::fs::File;
use zip::write::{FileOptions, ZipWriter};
use slugify::slugify;
use serenity::model::permissions::Permissions;

fn log_channel(channel_id: ChannelId, from_id: MessageId, until_id: MessageId) -> Result<File, CommandError> {
    let mut buf = tempfile::tempfile()?;
    let mut next_id = from_id;
    let mut next_ts = Date::from_utc(from_id.created_at().date(), Utc.fix());

    'outer: loop {
        let batch   = channel_id.messages(|m| m.after(next_id).limit(100))?;
        let count   = batch.len();
        let last_id = if let Some(msg) = batch.get(0) {
            msg.id
        } else {
            return Ok(buf);
        };

        for message in batch.into_iter().rev() {
            if message.timestamp.date() > next_ts {
                next_ts = message.timestamp.date();
                writeln!(&mut buf, "-- Day changed {}", next_ts.format("%A, %e %B %Y %Z"))?;
            }

            writeln!(&mut buf, "[{}] {}",
                     message.timestamp.format("%H:%M:%S"),
                     message.to_logstr())?;

            if message.id == until_id {
                break 'outer;
            }
        }

        if count < 100 || next_id == last_id {
            break;
        }

        next_id = last_id;
    }

    buf.seek(SeekFrom::Start(0))?;

    Ok(buf)
}

cmd!(LogChannel(_ctx, msg, args)
     aliases: ["log", "logs"],
     desc: "Generate a log file for a channel.",
     max_args: 3,
     required_permissions: Permissions::ADMINISTRATOR,
     usage: "[channel [from-msg-id [to-msg-id]]]`\nDefaults to the entirety of the current channel. `\u{200B}",
{
    let channel_id = match args.single::<String>() {
        Err(_) => msg.channel_id,
        Ok(s)  => {
            match serenity::utils::parse_channel(&s) {
                Some(id) => ChannelId(id),
                None     => s.parse::<u64>().map(ChannelId)?,
            }
        }
    };

    let from_id = match args.single::<String>() {
        Ok(s)  => s.trim().parse::<u64>().map(|i| MessageId(i - 1))?,
        Err(_) => MessageId(0),
    };

    let until_id = match args.single::<String>() {
        Ok(s)  => s.trim().parse::<u64>().map(MessageId)?,
        Err(_) => MessageId(std::u64::MAX),
    };

    let buf = tempfile::tempfile()?;
    let mut zip = ZipWriter::new(buf);

    let channel  = slugify!(&channel_id.name().unwrap_or_else(|| channel_id.to_string()));
    let filename = format!("{}-{}.txt", channel, msg.timestamp);
    let mut log  = log_channel(channel_id, from_id, until_id)?;

    zip.start_file(filename, FileOptions::default())?;
    std::io::copy(&mut log, &mut zip)?;

    let mut archive = zip.finish()?;
    archive.seek(SeekFrom::Start(0))?;

    let zipname = format!("{}-{}.zip", channel, msg.timestamp);
    let content = format!("Logs for {} from {} to {}.",
                          channel_id.mention(),
                          from_id.created_at(),
                          until_id.created_at());

    msg.channel_id.send_files(Some((&archive, &*zipname)), |m| m.reactions(Some('❌')).content(content))?;
});

cmd!(LogGuild(_ctx, msg, args)
     aliases: ["log_guild", "guild_logs"],
     desc: "Generate an archive of log files for a guild.",
     max_args: 1,
     required_permissions: Permissions::ADMINISTRATOR,
{
    let guild_id = match args.single::<String>() {
        Ok(s)  => s.trim().parse::<u64>().map(GuildId)?,
        Err(_) => msg.guild_id.ok_or("guild ID not found")?,
    };

    let buf = tempfile::tempfile()?;
    let mut zip = ZipWriter::new(buf);

    let guild = slugify!(&guild_id.to_guild_cached().ok_or("guild not found")?.read().name);
    zip.add_directory(guild.clone(), FileOptions::default())?;

    for channel_id in guild_id.channels()?.keys() {
        let channel  = slugify!(&channel_id.name().unwrap_or_else(|| channel_id.to_string()));
        let filename = format!("{}/{}-{}.txt", guild, channel, msg.timestamp);
        let mut log  = log_channel(*channel_id, MessageId(0), MessageId(std::u64::MAX))?;

        zip.start_file(filename, FileOptions::default())?;
        std::io::copy(&mut log, &mut zip)?;
    }

    let mut archive = zip.finish()?;
    archive.seek(SeekFrom::Start(0))?;

    let filename = format!("{}-{}.zip", guild, msg.timestamp);
    let content = format!("Log archive for {}.", guild);
    msg.channel_id.send_files(Some((&archive, &*filename)), |m| m.reactions(Some('❌')).content(content))?;
});

grp![LogChannel, LogGuild];
