use errors::*;

command!(calc(_ctx, msg, arg) {
    let expr = arg.join(" ");

    let result = ::rink::one_line_sandbox(expr);
    let output = format!("{}: {}", &msg.author.name, result);

    let _ = msg.channel_id.say(&output);
});

