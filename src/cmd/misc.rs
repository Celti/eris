use serenity::command;

command!(fnord(_ctx, msg) {
    msg.channel_id.say(&fnorder::fnorder())?;
});

command!(get_ddate(_ctx, msg) {
    use chrono::Utc;
    use ddate::DiscordianDate;

    msg.channel_id.say(&format!("Today is {}", Utc::today().to_poee()))?;
});
