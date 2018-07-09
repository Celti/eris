use serenity::command;

command!(set_playing(ctx, msg, args) {
    ctx.set_game_name(&args.full());
    msg.reply("Game set.")?;
});

command!(change_nick(_ctx, msg, args) {
    let guild = msg.guild_id().unwrap();
    let nick = args.full();

    guild.edit_nickname(match nick {
        n if n.is_empty() => None,
        n => Some(n)
    })?;
});
