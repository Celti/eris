use crate::types::*;
use serenity::command;
use std::process::Command;

command!(calc(_ctx, msg, args) {
    let expr = args.full().split(',');

    let output = Command::new("/usr/bin/units")
        .env("UNITS_ENGLISH", "US")
        .arg("--terse")
        .arg("--")
        .args(expr)
        .output()?;

    msg.reply(&String::from_utf8_lossy(&output.stdout))?;
});

command!(get_history(_ctx, msg, args) {
    use crate::util::MessageExt;
    use chrono::{Date, Offset, Utc};
    use std::io::{Seek, SeekFrom, Write};

    let channel_id = match args.single::<String>() {
        Err(_) => msg.channel_id,
        Ok(s) => {
            match serenity::utils::parse_channel(&s) {
                Some(id) => ChannelId(id),
                None => s.parse::<u64>().map(ChannelId)?,
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

    let mut buf = tempfile::tempfile()?;
    let mut next_id = from_id;
    let mut next_ts = Date::from_utc(from_id.created_at().date(), Utc.fix());

    'outer: loop {
        let batch   = channel_id.messages(|m| m.after(next_id).limit(100))?;
        let count   = batch.len();
        let last_id = batch[0].id;

        for message in batch.into_iter().rev() {
            // TODO improve formatting; e.g., embeds, attachments, emoji. HTML?
            if message.timestamp.date() > next_ts {
                next_ts = message.timestamp.date();
                writeln!(&mut buf, "\n\t{}\n", next_ts.format("%A, %e %B %Y %Z"));
            }

            writeln!(&mut buf, "{}", message.to_logstr())?;

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

    let channel  = channel_id.name().unwrap_or_else(|| channel_id.to_string());
    let filename = format!("{}-{}.txt", channel, msg.timestamp);

    let content = format!("Logs for {} from {} to {}.",
                          channel_id.mention(),
                          from_id.created_at(),
                          until_id.created_at());

    msg.channel_id.send_files(Some((&buf, &*filename)), |m| m.reactions(Some('❌')).content(content))?;
});

command!(get_timestamp(_ctx, msg, args) {
    // All snowflakes are the same for timestamps. ChannelId parses as desired.
    let id = args.single::<ChannelId>()?;
    let stamp = id.created_at();
    msg.reply(&format!("Snowflake {} was created at {} UTC.", id, stamp))?;
});
