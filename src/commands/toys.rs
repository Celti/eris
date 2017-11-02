command!(fnord(_ctx, msg) {
    msg.channel_id.say(&::fnorder::fnorder())?;
});
