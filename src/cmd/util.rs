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
    use crate::util::cached_display_name;
    use serenity::http::AttachmentType;
    use std::io::Write;

    let channel_id = args.single::<ChannelId>().or_else(|_| args.single::<u64>().map(ChannelId))?;
    let from_id    = MessageId(args.single::<u64>().unwrap_or(0));
    let until_id   = args.single::<u64>().map(MessageId).ok();

    let mut next_id  = from_id;
    let mut log_file = tempfile::tempfile()?;

    loop {
        let batch   = channel_id.messages(|m| m.after(next_id).limit(100))?;
        let last_id = batch.last().unwrap().id;

        for message in batch.into_iter() {
            writeln!(
                &mut log_file,
                "{} <{}> {}",
                message.timestamp,
                cached_display_name(message.channel_id, message.author.id)?,
                message.content)?;
        }

        if last_id == next_id {
            break;
        }

        if let Some(until_id) = until_id {
            if last_id > until_id {
                break;
            }
        }

        next_id = last_id;
    }

    let channel_name =
        if let Some(name) = channel_id.name() { name }
        else { channel_id.to_string() };

    let file_name = format!("{}-{}.txt", channel_name, msg.timestamp);
    let log_file = AttachmentType::File((&log_file, &file_name));

    msg.channel_id.send_files(vec![log_file], |m| m.content(""))?;
});

command!(get_timestamp(_ctx, msg, args) {
    // All snowflakes are the same for timestamps. ChannelId parses as desired.
    let id = args.single::<ChannelId>()?;
    let stamp = id.created_at();
    msg.reply(&format!("Snowflake {} was created at {} UTC.", id, stamp))?;
});
