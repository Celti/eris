command!(set_playing(ctx, msg, args) {
    ctx.set_game_name(&args.full());
    msg.reply("Game set.")?;
});

command!(quit(ctx, msg) {
    msg.channel_id.say("Goodnight, everybody!")?;
    ctx.quit()?;
});

command!(nick(_ctx, msg, args) {
    let guild = msg.guild_id().unwrap();
    let nick = args.full();

    guild.edit_nickname(match nick {
        ref n if n.is_empty() => None,
        ref n => Some(n.as_str())
    })?;
});

command!(prefix(ctx, msg, args) {
    use data::GuildPrefixes;
    use std::collections::HashMap;

    let mut data = ctx.data.lock();
    let mut map = data.entry::<GuildPrefixes>().or_insert(HashMap::default());

    let guild = msg.guild_id().unwrap();
    let prefix = args.full();

    map.insert(guild, prefix);
});
