use errors::*;
use rand::distributions::{IndependentSample, Range};
use regex::Regex;

pub struct Segment {
    rolls: i64,
    dice: i64,
    sides: i64,
    modifier: Option<String>,
    value: i64,
    versus: bool,
    tag: Option<String>,
    target: i64
}

lazy_static! {
    static ref RE: Regex = Regex::new(r"(?x)
        (?: (?P<rolls>\d+) [*x] )?                          # repeated rolls
        (?P<dice>\d+)? d (?P<sides>\d+)?                    # number of dice and sides
        (?: (?P<modifier>[-+*x/\\bw]) (?P<value>\d*) )?     # modifier and value
        (?: \s* (?: (?P<vs>vs?) \s*?                        # versus
            (?P<tag>\S+?.*?)? [\s-] )                           # tag
            (?P<target>-?\d+) )?                            # target
        \D?").unwrap();
}


fn parse_segments(expr: &str) -> Result<Vec<Segment>> {
    let mut segments: Vec<Segment> = Vec::new();

    for cap in RE.captures_iter(&expr) {
        segments.push( Segment {
            rolls: cap.name("rolls").map(|c| c.as_str()).unwrap_or("1").parse().unwrap(),
            dice: cap.name("dice").map(|c| c.as_str()).unwrap_or("3").parse().unwrap(),
            sides: cap.name("sides").map(|c| c.as_str()).unwrap_or("6").parse().unwrap(),
            modifier: cap.name("modifier").map(|c| c.as_str().to_string()),
            value: cap.name("value").map(|c| c.as_str()).unwrap_or("0").parse().unwrap(),
            versus: cap.name("vs").map(|c| c.as_str()).is_some(),
            tag: cap.name("tag").map(|c| c.as_str().to_string()),
            target: cap.name("target").map(|c| c.as_str()).unwrap_or("0").parse().unwrap(),
        } );
    }

    Ok(segments)
}

fn roll_dice(expr: &Segment) -> Result<String> {
    let mut rng = ::rand::thread_rng();
    let die = Range::new(1, expr.sides + 1);

    let mut rolls: Vec<i64> = Vec::new();

    for _ in 0 .. expr.dice {
        rolls.push(die.ind_sample(&mut rng));
    }

    let mut sum: i64 = rolls.iter().fold(0, |s,x| s + x);

    if let &Some(ref m) = &expr.modifier {
        match m.as_str() {
            "b" => { // Sum the best n, overwrite previous sum.
                rolls.sort_by(|a,b| b.cmp(a));
                sum = rolls[0 .. expr.value as usize - 1].iter().fold(0, |s,x| s + x);
            }
            "w" => { // Sum the worst n, overwrite previous sum.
                rolls.sort_by(|a,b| b.cmp(a));
                sum = rolls[0 .. expr.value as usize - 1].iter().fold(0, |s,x| s + x);
            }
            "+" => sum += expr.value,
            "-" => sum -= expr.value,
            "*"|"x" => sum *= expr.value,
            "/"|"\\" => sum /= expr.value,
            _ => unreachable!(),
        }
    }

    if expr.versus {
        let margin = expr.target - sum;
        let skill = format!("{}-{}", expr.tag.clone().unwrap_or("Skill".to_string()).trim(), expr.target);

        if sum < 5 || (expr.target > 14 && sum < 6) || (expr.target > 15 && sum < 7) {
            return Ok(format!("{} vs {}: Success by {} ***(CRITICAL SUCCESS)***", sum, skill, margin));
        } else if sum > 16 || margin <= -10 {
            if expr.target > 15 && sum == 17 {
                return Ok(format!("{} vs {}: Margin of {} **(Automatic Failure)**", sum, skill, margin));
            } else {
                return Ok(format!("{} vs {}: Failure by {} ***(CRITICAL FAILURE)***", sum, skill, margin.abs()));
            }
        } else if margin < 0 {
            return Ok(format!("{} vs {}: Failure by {}", sum, skill, margin.abs()));
        } else {
            return Ok(format!("{} vs {}: Success by {}", sum, skill, margin));
        }
    }

    if expr.modifier.is_some() {
        return Ok(format!("{}d{}{}{}: {} {:?}", expr.dice, expr.sides, expr.modifier.clone().unwrap(), expr.value, sum, rolls));
    } 

    Ok(format!("{}d{}: {} {:?}", expr.dice, expr.sides, sum, rolls))
}

command!(roll(_ctx, msg, arg) {
    let expr = arg.join(" ");

    let mut results: Vec<String> = Vec::new();

    let segments = parse_segments(&expr).map_err(stringify)?;

    for segment in segments {
        for _ in 0 .. segment.rolls {
            results.push(roll_dice(&segment).map_err(stringify)?);
        }
    }

    let output = format!("{}: {}\n {}", &msg.author.name, expr, results.join("\n") );

    let _ = msg.channel_id.say(&output);
});
