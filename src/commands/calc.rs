use rink::*;

fn eval(line: &str) -> String {
    let mut ctx = load().unwrap();
    ctx.short_output = true;
    match one_line(&mut ctx, line) {
        Ok(v) => v,
        Err(e) => e,
    }
}

command!(calc(_ctx, msg, args) {
    let _ = msg.reply(&eval(&args.full()));
});
