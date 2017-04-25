command!(fnord(_ctx, msg, _arg) {
    let _ = msg.channel_id.say(&::fnorder::fnorder());
});
