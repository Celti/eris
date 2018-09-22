// FIXME use_extern_macros
// use serenity::command;

command!(fnord(_ctx, msg) {
    msg.channel_id.say(&fnorder::fnorder())?;
});

command!(discdate(_ctx, msg) {
    use chrono::Utc;
    use ddate::DiscordianDate;

    msg.channel_id.say(&format!("Today is {}", Utc::today().to_poee()))?;
});
