command!(set_playing(ctx, msg, args) {
    ctx.set_game_name(&args.full());
    msg.reply("Game set.")?;
});

command!(quit(ctx, msg) {
    msg.channel_id.say("Goodnight, everybody!")?;
    ctx.quit();
});

command!(change_nick(_ctx, msg, args) {
    let guild = msg.guild_id().unwrap();
    let nick = args.full();

    guild.edit_nickname(match nick {
        n if n.is_empty() => None,
        n => Some(n)
    })?;
});

command!(change_guild_prefix(ctx, msg, args) {
    use data::GuildPrefixes;
    use std::collections::HashMap;

    let mut data = ctx.data.lock();
    let mut map = data.entry::<GuildPrefixes>().or_insert(HashMap::default());

    let guild = msg.guild_id().unwrap();
    let prefix = args.full().to_string();

    map.insert(guild, prefix);
});

command!(get_history(_ctx, msg, _args) {
    use std::fmt::Write;
    use serenity::http::AttachmentType;
    use serenity::model::channel::Channel;
    use utils;

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
            utils::cached_display_name(message.channel_id, message.author.id)?,
            message.content)?;
    }

    let channel_name = match msg.channel_id.get()? {
        Channel::Guild(c) => {
            format!("{}-{}",
                c.read().guild_id.get()?.name,
                c.read().name.clone())
        }
        Channel::Group(c) => c.read().name.clone().unwrap_or(msg.channel_id.to_string()),
        Channel::Private(c) => c.read().recipient.read().name.clone(),
        Channel::Category(_) => unreachable!(),
    };

    let file_name = format!("{}-{}.txt", channel_name, msg.timestamp);
    let log_file = AttachmentType::Bytes((buf.as_bytes(), &file_name));

    msg.channel_id.send_files(vec![log_file], |m| m.content(""))?;
});
