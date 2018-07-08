command!(fnord(_ctx, msg) {
    msg.channel_id.say(&::fnorder::fnorder())?;
});

command!(ddate(_ctx, msg) {
    use chrono::Utc;
    use ddate::DiscordianDate;

    msg.channel_id.say(&format!("Today is {}", Utc::today().to_poee()))?;
});
