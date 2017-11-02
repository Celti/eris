use rink::{load, one_line};

command!(calc(_ctx, msg, args) {
    let mut ctx = load()?;
    ctx.short_output = true;

    let output = match one_line(&mut ctx, &args.full()) {
        Ok(r) => r, Err(e) => e
    };

    msg.reply(&output)?;
});
