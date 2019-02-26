use rand::seq::SliceRandom;
use rand::thread_rng;
use rand::Rng;
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult, Delimiter};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;

const ANSWERS: [&str; 28] = [
    "Yes.",
    "My sources say yes.",
    "As I see it, yes.",
    "Of course!",
    "Ha! What a dumb question! Yes.",
    "No.",
    "My sources say no.",
    "Maybe, but don't count on it.",
    "Hell no!",
    "Ha! What a dumb question! No.",
    "Maybe.",
    "How the hell should I know?",
    "Only under certain conditions.",
    "I have no idea!",
    "Hm. That's a very good question. Maybe?",
    "Can I lie about the answer?",
    "Go flip a coin!",
    "I don't think I should answer that.",
    "I'm in a bad mood, go away.",
    "If I told you that, I'd have to kill you.",
    "My lawyer says I shouldn't answer that on the grounds that I may incriminate myself.",
    "My sources are mysteriously silent on that subject.",
    "Once in a blue moon.",
    "That is a question you should ask yourself.",
    "Why do you want to know?",
    "Corner pocket.",
    "Scratch.",
    "Side pocket.",
];

#[command]
#[aliases(decide, pick)]
#[description("Choose from amongst multiple options.")]
#[min_args(2)]
fn choose(ctx: &mut Context, msg: &Message, args: Args) -> CommandResult {
    let delimiters = [
        Delimiter::Multiple(String::from(", or ")),
        Delimiter::Multiple(String::from(", ")),
        Delimiter::Single(','),
        Delimiter::Multiple(String::from(" or "))
    ];

    let mut args = Args::new(args.message(), &delimiters);
    let choices = args.iter::<String>().collect::<Result<Vec<_>,_>>()?;

    // let choices = args.full().split(", or ")
    //     .flat_map(|s| s.split(", "))
    //     .flat_map(|s| s.split(','))
    //     .flat_map(|s| s.split(" or "))
    //     .collect::<Vec<_>>();

    if choices.len() < 2 {
        reply!(ctx, msg, "No.");
    } else {
        reply!(ctx, msg, "{}", choices.choose(&mut thread_rng()).unwrap());
    }

    Ok(())
}

#[command]
#[aliases(coin)]
#[description("Flip a coin.")]
#[num_args(0)]
fn flip(ctx: &mut Context, msg: &Message) -> CommandResult {
    let mut rng = rand::thread_rng();
    msg.reply(&ctx, {
        if rng.gen_bool(0.01) {
            "Edge!"
        } else if rng.gen() {
            "Heads!"
        } else {
            "Tails!"
        }
    })?;

    Ok(())
}

#[command]
#[aliases("eight","8ball")]
#[description("Ask the Magic 8-Ball™ a question.")]
#[min_args(1)]
fn ask(ctx: &mut Context, msg: &Message) -> CommandResult {
    reply!(ctx, msg, "{}", ANSWERS.choose(&mut thread_rng()).unwrap());

    Ok(())
}

group!({
    name: "random",
    options: {},
    commands: [ask, choose, flip]
});
