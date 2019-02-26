use chrono::Utc;
use ddate::DiscordianDate;
use serenity::client::Context;
use serenity::framework::standard::CommandResult;
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;

#[command]
#[description("Receive a message from the Conspiracy.")]
fn fnord(ctx: &mut Context, msg: &Message) -> CommandResult {
    say!(ctx, msg, "{}", fnorder::fnorder());

    Ok(())
}

#[command]
#[description("PERPETUAL DATE CONVERTER FROM GREGORIAN TO POEE CALENDAR")]
fn ddate(ctx: &mut Context, msg: &Message) -> CommandResult {
    say!(ctx, msg, "Today is {}", Utc::today().to_poee());

    Ok(())
}

group!({
    name: "toys",
    options: {},
    commands: [ddate, fnord]
});
