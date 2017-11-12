use ext::dice::DiceVec;
use rand::{self, Rng};
use serenity::model::*;
use std::collections::HashMap;
use std::str::FromStr;
use failure::Error;
use utils;
use data::DiceMessages;

pub fn roll_and_send(
    map: &mut HashMap<MessageId, DiceVec>,
    channel_id: ChannelId,
    user_id: UserId,
    set: DiceVec,
) -> Result<Message, Error> {
    let name = utils::cached_display_name(channel_id, user_id)?;
    let sent = channel_id.send_message(|m| {
        m.content(format!(
            "**{} rolled {}:**\n```\n{}\n```",
            name,
            set.to_string(),
            set.roll().join("\n")
        )).reactions(vec!['🎲'])
    })?;

    map.insert(sent.id, set);

    Ok(sent)
}

command!(roll(ctx, msg, args) {
    let mut expr = args.full();

    let set = if let Ok(dice) = DiceVec::from_str(&expr) { dice } else {
        // Ugly hack to retry failed parsing prefixed with '3d6'.
        // This allows, e.g. `!roll vs 10` to parse as `!roll 3d6 vs 10`
        expr = "3d6 ".to_string() + &expr;
        DiceVec::from_str(&expr).unwrap()
    };

    let mut data = ctx.data.lock();
    let mut map = data.entry::<DiceMessages>().or_insert(HashMap::default());

    roll_and_send(map, msg.channel_id, msg.author.id, set)?;
});

command!(flip(_ctx, msg) {
    let mut rng = rand::thread_rng();

    msg.reply(if rng.gen_weighted_bool(1000) {
        "Edge!"
    } else if rng.gen() {
        "Heads!"
    } else {
        "Tails!"
    })?;
});

command!(choose(_ctx, msg, args) {
    let choices = args.list::<String>()?;
    msg.reply(rand::thread_rng().choose(&choices)
        .unwrap_or(&String::from("do you really need me for that decision?")))?;
});

command!(eight(_ctx, msg) {
    const ANSWERS: [&str; 28] = [
        "Yes.", "My sources say yes.", "As I see it, yes.", "Of course!",
        "Ha! What a dumb question! Yes.", "No.", "My sources say no.",
        "Maybe, but don't count on it.", "Hell no!", "Ha! What a dumb question! No.",
        "Maybe.", "How the hell should I know?", "Only under certain conditions.",
        "I have no idea!", "Hm. That's a very good question. Maybe?",
        "Can I lie about the answer?", "Go flip a coin!",
        "I don't think I should answer that.", "I'm in a bad mood, go away.",
        "If I told you that, I'd have to kill you.",
        "My lawyer says I shouldn't answer that on the grounds that I may incriminate myself.",
        "My sources are mysteriously silent on that subject.", "Once in a blue moon.",
        "That is a question you should ask yourself.",
        "Why do you want to know?", "Corner pocket.", "Scratch.", "Side pocket."
    ];

    msg.reply(rand::thread_rng().choose(&ANSWERS).unwrap())?;
});
