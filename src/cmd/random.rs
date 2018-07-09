use serenity::command;
use rand::Rng;

command!(flip(_ctx, msg) {
    let mut rng = rand::thread_rng();

    msg.reply(if rng.gen_bool(0.01) {
        "Edge!"
    } else if rng.gen() {
        "Heads!"
    } else {
        "Tails!"
    })?;
});

command!(choose(_ctx, msg, args) {
    let choices = args.full().split(", or ")
        .flat_map(|s| s.split(", "))
        .flat_map(|s| s.split(","))
        .flat_map(|s| s.split(" or "))
        .collect::<Vec<_>>();

    if choices.len() < 2 {
        msg.reply("No.")?;
    } else {
        msg.reply(rand::thread_rng().choose(&choices).unwrap())?;
    }
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

