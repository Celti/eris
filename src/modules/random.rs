use rand::Rng;
use rand::seq::SliceRandom;
use rand::thread_rng;

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

cmd!(Choose(_ctx, msg, args)
     aliases: ["choose", "decide", "pick"],
     desc: "Choose between multiple comma-delimited options.",
     example: "Option A, Option B, or Option C",
     min_args: 2,
{
    let choices = args.full().split(", or ")
        .flat_map(|s| s.split(", "))
        .flat_map(|s| s.split(','))
        .flat_map(|s| s.split(" or "))
        .collect::<Vec<_>>();

    if choices.len() < 2 {
        msg.reply("No.")?;
    } else {
        msg.reply(choices.choose(&mut thread_rng()).unwrap())?;
    }
});

cmd!(CoinFlip(_ctx, msg)
     aliases: ["flip", "coin"],
     desc: "Flips a coin.",
     num_args: 0,
{
    let mut rng = rand::thread_rng();

    msg.reply(if rng.gen_bool(0.01) {
        "Edge!"
    } else if rng.gen() {
        "Heads!"
    } else {
        "Tails!"
    })?;
});

cmd!(EightBall(_ctx, msg)
     aliases: ["8ball", "ask", "eight"],
     desc: "Ask the Magic 8-Ball™ a yes-or-no question.",
     min_args: 1,
{
        msg.reply(ANSWERS.choose(&mut thread_rng()).unwrap())?;
});

grp![Choose, CoinFlip, EightBall];
