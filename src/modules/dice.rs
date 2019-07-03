use crate::ext::dice::DiceRoll;
use crate::model::DiceCache;
use itertools::Itertools;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;

pub fn handle_roll(ctx: &Context, channel: ChannelId, user: UserId, input: &str) {
    let (expr, comment) = {
        if let Some((expr, comment)) = input.splitn(2, '#').collect_tuple() {
            (expr, format!(" _{}_", comment.trim()))
        } else {
            (input, String::new())
        }
    };

    let roll = expr
        .split(|c| c == ';' || c == '\n')
        .map(|s| if s.is_empty() { "3d6" } else { s })
        .filter_map(|s| s.parse::<DiceRoll>().ok())
        .map(|r| r.to_string())
        .join("\n");

    let content = format!("**{} rolled:**{}\n```{}```", user.mention(), comment, roll);

    match channel.send_message(&ctx, |m| m.content(content).reactions(Some('🎲'))) {
        Err(err) => log::warn!("[{}:{}] {:?}", line!(), column!(), err),
        Ok(msg) => {
            let mut data = ctx.data.write();
            let cache = data.entry::<DiceCache>().or_insert_with(Default::default);
            cache.insert(msg.id, input.to_string());
        }
    }
}

#[command]
#[description("Calculate an expression in modified dice notation.")]
#[usage("[expr][; expr...]`\nFor details, see https://github.com/Celti/eris/wiki/Dice-Expressions `\u{200B}")]
fn roll(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    handle_roll(ctx, msg.channel_id, msg.author.id, args.message());

    Ok(())
}

group!({
    name: "dice",
    options: {},
    commands: [roll]
});
