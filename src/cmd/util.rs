// FIXME use_extern_macros
// use serenity::command;

use serenity::model::id::MessageId;
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

command!(get_history(_ctx, msg, _args) {
    use std::fmt::Write;
    use serenity::http::AttachmentType;
    use crate::util::cached_display_name;

    let mut last = msg.id;
    let mut messages = Vec::new();
    let mut buf = String::new();

    loop {
        let mut batch = msg.channel_id.messages(|m| m.before(last).limit(100))?;

        messages.append(&mut batch);

        let next_id = messages[messages.len() - 1].id;

        if next_id == last {
            break;
        }

        last = next_id;
    }

    for message in messages.into_iter().rev() {
        writeln!(
            &mut buf,
            "{} <{}> {}",
            message.timestamp,
            cached_display_name(message.channel_id, message.author.id)?,
            message.content)?;
    }

    let channel_name =
        if let Some(name) = msg.channel_id.name() { name }
        else { msg.channel_id.to_string() };

    let file_name = format!("{}-{}.txt", channel_name, msg.timestamp);
    let log_file = AttachmentType::Bytes((buf.as_bytes(), &file_name));

    msg.channel_id.send_files(vec![log_file], |m| m.content(""))?;
});

command!(get_timestamp(_ctx, msg, args) {
    let message = MessageId(args.single::<u64>()?);
    let stamp = message.created_at();
    msg.reply(&format!("Snowflake {} was created at {} UTC.", message, stamp))?;
});
