use rand::{self,Rng};
use rand::distributions::{IndependentSample, Range};
use regex::Regex;

struct DiceExpr {
    rolls: i64,
    dice: i64,
    sides: i64,
    modifier: Option<String>,
    value: i64,
    versus: bool,
    tag: String,
    target: i64,
}

impl DiceExpr {
    fn roll(&self) -> String {
        let mut rng = rand::thread_rng();
        let mut rolls: Vec<i64> = Vec::new();
        let die = Range::new(1, self.sides + 1);

        for _ in 0..self.dice {
            rolls.push(die.ind_sample(&mut rng));
        }

        let mut sum: i64 = rolls.iter().fold(0, |s, x| s + x);

        if let Some(ref m) = self.modifier {
            match m.as_str() {
                "b" => {
                    rolls.sort_by(|a, b| b.cmp(a));
                    sum = rolls[0..self.value as usize].iter().fold(0, |s, x| s + x);
                }
                "w" => {
                    rolls.sort_by(|a, b| a.cmp(b));
                    sum = rolls[0..self.value as usize].iter().fold(0, |s, x| s + x);
                }
                "+" => sum += self.value,
                "-" => sum -= self.value,
                "*" | "x" | "×" => sum *= self.value,
                "/" | "\\" | "÷" => sum /= self.value,
                _ => unreachable!(),
            }
        }

        if self.versus && self.dice == 3 && self.sides == 6 {
            // GURPS 4th Edition success roll.
            let margin = self.target - sum; // Roll under.
            let skill = format!("{}-{}", self.tag.trim(), self.target);

            if sum < 5 || (self.target > 14 && sum < 6) || (self.target > 15 && sum < 7) {
                format!("{:>2} vs {}: Success by {} (CRITICAL SUCCESS)", sum, skill, margin)
            } else if sum > 16 || margin <= -10 {
                if self.target > 15 && sum == 17 {
                    format!("{:>2} vs {}: Margin of {} (Automatic Failure)", sum, skill, margin)
                } else {
                    format!("{:>2} vs {}: Failure by {} (CRITICAL FAILURE)", sum, skill, margin.abs())
                }
            } else if margin < 0 {
                format!("{:>2} vs {}: Failure by {}", sum, skill, margin.abs())
            } else {
                format!("{:>2} vs {}: Success by {}", sum, skill, margin)
            }
        } else if self.versus && self.dice == 1 && self.sides == 20 {
            // Generic d20 system success roll.
            let margin = sum - self.target; // Roll over.
            let skill = format!("{}-{}", self.tag.trim(), self.target);

            if margin < 0 {
                format!("{:>2} vs {}: Failure by {}", sum, skill, margin.abs())
            } else {
                format!("{:>2} vs {}: Success by {}", sum, skill, margin)
            }
        } else if self.versus && self.dice == 1 && self.sides == 100 {
            // Generic percentile system success roll.
            let margin = self.target - sum; //Roll under.
            let skill = format!("{}-{}", self.tag.trim(), self.target);

            if margin < 0 {
                format!("{:>2} vs {}: Failure by {}", sum, skill, margin.abs())
            } else {
                format!("{:>2} vs {}: Success by {}", sum, skill, margin)
            }
        } else if let Some(ref modifier) = self.modifier {
            // Not a versus roll, output with modifier.
            format!("{}d{}{}{}: {:>3} {:?}",
                self.dice, self.sides, modifier, self.value, sum, rolls)
        } else {
            // Bog-standard normal die roll.
            format!("{}d{}: {:>3} {:?}", self.dice, self.sides, sum, rolls)
        }
    }
}

fn parse_dice(expr: &str) -> Vec<DiceExpr> {
    lazy_static! { static ref RE: Regex = Regex::new(r"(?x)
        (?: (?P<rolls>\d+) [*x] )?                         # repeated rolls
        (?: (?P<dice>\d+) d (?P<sides>\d+)? )              # number, optional sides
        (?: (?P<modifier>[-+*x×÷/\\bw]) (?P<value>\d*) )?  # modifier and value
        (?: \s* (?: (?P<vs>vs?) \s*?                       # versus
            (?P<tag>\S+?.*?)? [\s-] )                      # tag
            (?P<target>-?\d+) )?                           # target
        ").unwrap();
    }

    let mut dice: Vec<DiceExpr> = Vec::new();

    for cap in RE.captures_iter(expr) {
        dice.push(DiceExpr {
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

    dice
}

command!(roll(_ctx, msg, args) {
    let mut expr = args.full();
    let mut results: Vec<String> = Vec::new();

    let mut segments = parse_dice(&expr);

    if segments.is_empty() {
        expr = "3d6 ".to_string() + &expr;
        segments = parse_dice(&expr);
    }

    for segment in segments {
        for _ in 0 .. segment.rolls {
            results.push(segment.roll());
        }
    }

    let name = if let Some(guild) = msg.guild_id() {
        guild.member(msg.author.id)?.display_name().into_owned()
    } else {
        msg.author.name.clone()
    };

    msg.channel_id.send_message(|m| m.embed(|e| e
        .author(|a| a
            .name(&format!("@{} rolled {}", &name, &expr))
            .icon_url(&msg.author.face()))
        .colour(0xFF_D7_00)
        .description(&format!("```\n{}\n```", &results.join("\n")))
    ))?;
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
    let mut rng = rand::thread_rng();
    let choices = args.list::<String>()?;
    msg.reply(rng.choose(&choices).unwrap())?;
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

    let mut rng = rand::thread_rng();
    msg.reply(rng.choose(&ANSWERS).unwrap())?;
});
