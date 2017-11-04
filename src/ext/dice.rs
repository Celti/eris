use std::fmt::{self, Display};
use std::str::FromStr;
use std::string::ToString;

#[derive(Clone, Debug)]
pub struct DiceExpr {
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
    fn roll_once(&self) -> String {
        use rand::distributions::{IndependentSample, Range};

        let mut rng = ::rand::thread_rng();
        let die = Range::new(1, self.sides + 1);

        let mut rolls: Vec<i64> = Vec::with_capacity(self.dice as usize);

        for _ in 0..self.dice {
            rolls.push(die.ind_sample(&mut rng));
        }

        let mut sum = rolls.iter().fold(0, |s, x| s + x);

        if let Some(ref m) = self.modifier {
            match m.as_str() {
                "b" => {
                    let index = ::std::cmp::min(self.value, self.dice) as usize;
                    rolls.sort_by(|a, b| b.cmp(a));
                    sum = rolls[0..index].iter().fold(0, |s, x| s + x);
                }
                "w" => {
                    let index = ::std::cmp::min(self.value, self.dice) as usize;
                    rolls.sort_by(|a, b| a.cmp(b));
                    sum = rolls[0..index].iter().fold(0, |s, x| s + x);
                }
                "+" => sum += self.value,
                "-" => sum -= self.value,
                "*" | "x" | "×" => sum *= self.value,
                "/" | "\\" | "÷" => sum = sum.checked_div(self.value).unwrap_or(0),
                _ => unreachable!(),
            }
        }

        if self.versus && self.dice == 3 && self.sides == 6 {
            // GURPS 4th Edition success roll.
            let margin = self.target - sum; // Roll under.
            let skill = format!("{}-{}", self.tag.trim(), self.target);

            if sum < 5 || (self.target > 14 && sum < 6) || (self.target > 15 && sum < 7) {
                format!(
                    "{:>2} vs {}: Success by {} (CRITICAL SUCCESS)",
                    sum,
                    skill,
                    margin
                )
            } else if sum > 16 || margin <= -10 {
                if self.target > 15 && sum == 17 {
                    format!(
                        "{:>2} vs {}: Margin of {} (Automatic Failure)",
                        sum,
                        skill,
                        margin
                    )
                } else {
                    format!(
                        "{:>2} vs {}: Failure by {} (CRITICAL FAILURE)",
                        sum,
                        skill,
                        margin.abs()
                    )
                }
            } else if margin < 0 {
                format!("{:>2} vs {}: Failure by {}", sum, skill, margin.abs())
            } else {
                format!("{:>2} vs {}: Success by {}", sum, skill, margin)
            }
        } else if self.versus && self.sides == 20 &&
                   (self.dice == 1 ||
                        (self.dice == 2 && self.value == 1 &&
                             (matches!(self.modifier, Some(ref x) if x == "b") ||
                                  matches!(self.modifier, Some(ref x) if x == "w"))))
        {
            // Generic d20 system success roll.
            let margin = sum - self.target; // Roll over.
            let skill = format!("{}{}", self.tag.trim(), self.target);

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
                format!("{:>3} vs {}: Failure by {}", sum, skill, margin.abs())
            } else {
                format!("{:>3} vs {}: Success by {}", sum, skill, margin)
            }
        } else {
            format!("{}: {:>3} {:?}", self, sum, rolls)
        }
    }

    fn roll(&self) -> Vec<String> {
        let mut results = Vec::with_capacity(self.rolls as usize);

        for _ in 0..self.rolls {
            results.push(self.roll_once());
        }

        results
    }
}

impl Display for DiceExpr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.rolls > 1 {
            write!(f, "{}x", self.rolls)?;
        }

        write!(f, "{}d{}", self.dice, self.sides)?;

        if let Some(ref m) = self.modifier {
            write!(f, "{}{}", m, self.value)?;
        }

        if self.versus {
            write!(f, " vs {}-{}", self.tag.trim(), self.target)?;
        }

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct DiceVec(Vec<DiceExpr>);

impl DiceVec {
    pub fn roll(&self) -> Vec<String> {
        self.0.iter().flat_map(|e| e.roll()).collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct ParseDiceError;

impl Display for ParseDiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "provided string did not contain a valid dice expression".fmt(f)
    }
}

impl FromStr for DiceVec {
    type Err = ParseDiceError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use regex::Regex;

        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?x)
                (?: (?P<rolls>\d+) [*x] )?                         # repeated rolls
                (?: (?P<dice>\d+) d (?P<sides>\d+)? )              # number, optional sides
                (?: (?P<modifier>[-+*x×÷/\\bw]) (?P<value>\d+) )?  # modifier and value
                (?: \s* (?: (?P<vs>vs?) \s*?                       # versus
                    (?P<tag>\S+?.*?)? [\s-] )                      # tag
                    (?P<target>-?\d+) )?                           # target
                ").unwrap();
        }

        let mut dice: DiceVec = DiceVec(Vec::new());

        for cap in RE.captures_iter(s) {
            dice.0.push(DiceExpr {
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

        if dice.0.is_empty() {
            Err(ParseDiceError)
        } else {
            Ok(dice)
        }
    }
}

impl Display for DiceVec {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join(", ")
            .fmt(f)
    }
}
