command!(set_playing(ctx, msg, args) {
    ctx.set_game_name(&args.full());
    msg.reply("Game set.")?;
});

command!(quit(ctx, msg) {
    let data = ctx.data.lock();
    let shard_manager = data.get::<::key::ShardManager>()
        .expect("ShardManager not present in Context::data");

    msg.channel_id.say("Goodnight, everybody!")?;
    shard_manager.lock().shutdown_all();
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
    use diesel::{self, prelude::*};
    use diesel::pg::upsert::excluded;
    use schema::guilds::dsl::*;
    use key::{DatabaseConnection, PrefixCache};

    let set_id = msg.guild_id().unwrap();
    let set_prefix = args.full().to_string();

    let mut data = ctx.data.lock();
    let cache = data.get_mut::<PrefixCache>().unwrap();
    cache.insert(set_id, set_prefix.clone());

    let db = data.get::<DatabaseConnection>().unwrap();
    diesel::insert_into(guilds)
        .values((guild_id.eq(set_id.0 as i64), &prefix.eq(set_prefix)))
        .on_conflict(guild_id).do_update()
        .set(prefix.eq(excluded(prefix)))
        .execute(&*db.lock())?;
});

command!(get_history(_ctx, msg, _args) {
    use std::fmt::Write;
    use serenity::http::AttachmentType;
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

    let channel_name =
        if let Some(name) = msg.channel_id.name() { name }
        else { msg.channel_id.to_string() };

    let file_name = format!("{}-{}.txt", channel_name, msg.timestamp);
    let log_file = AttachmentType::Bytes((buf.as_bytes(), &file_name));

    msg.channel_id.send_files(vec![log_file], |m| m.content(""))?;
});
