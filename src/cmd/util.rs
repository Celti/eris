// FIXME use_extern_macros
// use serenity::command;

use serenity::model::id::{ChannelId, MessageId};
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
    // TODO: Make this use a temporary file to prevent memory exhaustion
    // TODO: Add a deletion reaction

    use crate::util::cached_display_name;
    use serenity::http::AttachmentType;
    use std::io::Write;

    let channel_id = match args.current() {
        None => msg.channel_id,
        Some(s) => {
            match serenity::utils::parse_channel(s) {
                Some(id) => ChannelId(id),
                None => s.parse::<u64>().map(ChannelId)?,
            }
        }
    };

    let from_id  = MessageId(args.single::<u64>().unwrap_or(0));
    let until_id = MessageId(args.single::<u64>().unwrap_or(std::u64::MAX));

    let mut next_id  = from_id;
    let mut log_file = Vec::new();

    loop {
        let batch   = channel_id.messages(|m| m.after(next_id).limit(100))?;
        let count   = batch.len();
        let last_id = batch[0].id;

        for message in batch.into_iter().rev() {
            writeln!(
                log_file,
                "{} <{}> {}",
                message.timestamp,
                cached_display_name(message.channel_id, message.author.id)?,
                message.content
            )?;
        }

        log_file.flush()?;

        if count < 100 || last_id == next_id || last_id >= until_id {
            break;
        }

        next_id = last_id;
    }


    let channel_name = channel_id.name().unwrap_or_else(|| channel_id.to_string());
    let file_name    = format!("{}-{}.txt", channel_name, msg.timestamp);
    let attachment = AttachmentType::Bytes((&log_file, &file_name));

    msg.channel_id.send_files(vec![attachment], |m| m.content(""))?;
});

command!(get_timestamp(_ctx, msg, args) {
    // All snowflakes are the same for timestamps. ChannelId parses as desired.
    let id = args.single::<ChannelId>()?;
    let stamp = id.created_at();
    msg.reply(&format!("Snowflake {} was created at {} UTC.", id, stamp))?;
});
