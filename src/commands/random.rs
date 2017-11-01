use rand::Rng;
use rand::distributions::{IndependentSample, Range};
use regex::Regex;

pub struct Segment {
    rolls: i64,
    dice: i64,
    sides: i64,
    modifier: Option<String>,
    value: i64,
    versus: bool,
    tag: String,
    target: i64,
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"(?x)
        (?: (?P<rolls>\d+) [*x] )?                         # repeated rolls
        (?: (?P<dice>\d+) d (?P<sides>\d+)? )              # number and sides
        (?: (?P<modifier>[-+*x×÷/\\bw]) (?P<value>\d*) )?  # modifier and value
        (?: \s* (?: (?P<vs>vs?) \s*?                       # versus
            (?P<tag>\S+?.*?)? [\s-] )                      # tag
            (?P<target>-?\d+) )?                           # target
        ").unwrap();
}

fn parse_segments(expr: &str) -> Vec<Segment> {
    let mut segments: Vec<Segment> = Vec::new();

    for cap in RE.captures_iter(expr) {
        segments.push(Segment {
            rolls: cap.name("rolls")
                .map(|c| c.as_str())
                .unwrap_or("1")
                .parse()
                .unwrap(),
            dice: cap.name("dice")
                .map(|c| c.as_str())
                .unwrap_or("3")
                .parse()
                .unwrap(),
            sides: cap.name("sides")
                .map(|c| c.as_str())
                .unwrap_or("6")
                .parse()
                .unwrap(),
            modifier: cap.name("modifier").map(|c| c.as_str().to_string()),
            value: cap.name("value")
                .map(|c| c.as_str())
                .unwrap_or("0")
                .parse()
                .unwrap(),
            versus: cap.name("vs").map(|c| c.as_str()).is_some(),
            tag: cap.name("tag")
                .map(|c| c.as_str())
                .unwrap_or("Skill")
                .to_string(),
            target: cap.name("target")
                .map(|c| c.as_str())
                .unwrap_or("0")
                .parse()
                .unwrap(),
        });
    }

    segments
}

fn roll_dice(expr: &Segment) -> String {
    let mut rng = ::rand::thread_rng();
    let die = Range::new(1, expr.sides + 1);

    let mut rolls: Vec<i64> = Vec::new();

    for _ in 0..expr.dice {
        rolls.push(die.ind_sample(&mut rng));
    }

    let mut sum: i64 = rolls.iter().fold(0, |s, x| s + x);

    if let Some(ref m) = expr.modifier {
        match m.as_str() {
            "b" => {
                rolls.sort_by(|a, b| b.cmp(a));
                sum = rolls[0..expr.value as usize].iter().fold(0, |s, x| s + x);
            }
            "w" => {
                rolls.sort_by(|a, b| a.cmp(b));
                sum = rolls[0..expr.value as usize].iter().fold(0, |s, x| s + x);
            }
            "+" => sum += expr.value,
            "-" => sum -= expr.value,
            "*" | "x" | "×" => sum *= expr.value,
            "/" | "\\" | "÷" => sum /= expr.value,
            _ => unreachable!(),
        }
    }

    if expr.versus && expr.dice == 3 && expr.sides == 6 {
        let margin = expr.target - sum;
        let skill = format!("{}-{}", expr.tag.trim(), expr.target);

        if sum < 5 || (expr.target > 14 && sum < 6) || (expr.target > 15 && sum < 7) {
            return format!(
                "{:>2} vs {}: Success by {} (CRITICAL SUCCESS)",
                sum,
                skill,
                margin
            );
        } else if sum > 16 || margin <= -10 {
            if expr.target > 15 && sum == 17 {
                return format!(
                    "{:>2} vs {}: Margin of {} (Automatic Failure)",
                    sum,
                    skill,
                    margin
                );
            } else {
                return format!(
                    "{:>2} vs {}: Failure by {} (CRITICAL FAILURE)",
                    sum,
                    skill,
                    margin.abs()
                );
            }
        } else if margin < 0 {
            return format!("{:>2} vs {}: Failure by {}", sum, skill, margin.abs());
        } else {
            return format!("{:>2} vs {}: Success by {}", sum, skill, margin);
        }
    }

    if let Some(ref modifier) = expr.modifier {
        format!(
            "{}d{}{}{}: {:>3} {:?}",
            expr.dice,
            expr.sides,
            modifier,
            expr.value,
            sum,
            rolls
        )
    } else {
        format!("{}d{}: {:>3} {:?}", expr.dice, expr.sides, sum, rolls)
    }
}

command!(roll(_ctx, msg, args) {
    let mut expr = args.full();
    let mut results: Vec<String> = Vec::new();

    let mut segments = parse_segments(&expr);

    if segments.is_empty() {
        expr = "3d6 ".to_string() + &expr;
        segments = parse_segments(&expr);
    }

    for segment in segments {
        for _ in 0 .. segment.rolls {
            results.push(roll_dice(&segment));
        }
    }

    let name = if let Some(guild) = msg.guild_id() {
        guild.member(msg.author.id)?.display_name().into_owned()
    } else {
        msg.author.name.clone()
    };

    #[allow(unreadable_literal)]
    let _ = msg.channel_id.send_message(|m| m.embed(|e| e
        .author(|a| a
            .name(&format!("@{} rolled {}", &name, &expr))
            .icon_url(&msg.author.face()))
        .colour(0xFFD700)
        .description(&format!("```\n{}\n```", &results.join("\n")))
    ));
});

command!(flip(_ctx, msg) {
    let mut rng = ::rand::thread_rng();

    let _ = if rng.gen_weighted_bool(1000) {
        msg.reply("Edge!")
    } else if rng.gen() {
        msg.reply("Heads!")
    } else {
        msg.reply("Tails!")
    };
});

command!(choose(_ctx, msg, args) {
    let mut rng = ::rand::thread_rng();
    let choices = args.list::<String>()?;
    let _ = msg.reply(rng.choose(&choices).unwrap());
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

    let mut rng = ::rand::thread_rng();
    let _ = msg.reply(rng.choose(&ANSWERS).unwrap());
});
