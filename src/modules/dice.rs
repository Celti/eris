use crate::model::DiceCache;
use crate::modules::dice::parse::Rolls;

use itertools::Itertools;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;

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

pub fn handle_roll(ctx: &Context, channel: ChannelId, user: UserId, input: &str) {
    let (expr, comment) = {
        if let Some((expr, comment)) = input.splitn(2, '#').collect_tuple() {
            (expr, comment)
        } else {
            (input, "")
        }
    };

    let roll = expr.split(|c| c == ';' || c == '\n')
                   .map(|s| if s.is_empty() { "3d6" } else { s })
                   .filter_map(|s| s.parse::<Rolls>().ok())
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

pub mod parse {
    use lazy_static::lazy_static;
    use rand::distributions::{Distribution, Uniform};
    use regex::Regex;
    use std::fmt::{Display, Formatter, Result as FmtResult};
    use std::str::FromStr;
    use std::{error::Error, num::ParseIntError};

    #[derive(Clone, Debug)]
    pub struct Rolls(Vec<Roll>);

    #[derive(Clone, Debug)]
    struct Roll {
        terms: Vec<(Term, Vec<isize>)>,
        total: isize,
        versus: Option<(isize, Option<String>)>,
    }

    #[derive(Clone, Debug)]
    enum Term {
        Dice {
            n: usize,
            s: usize,
            t: Option<isize>,
        },
        Versus {
            s: isize,
            t: Option<String>,
        },
        Repeat(usize),
        Num(isize),
        Add,
        Sub,
        Mul,
        Div,
        Rem,
        Pow,
    }

    impl Rolls {
        fn new(t: &[Term]) -> Self {
            let mut rolls = Vec::new();
            let mut repeat = 1;
            let mut roll = false;

            for term in t.iter() {
                if let Term::Repeat(i) = term {
                    repeat = *i;
                } else if let Term::Dice { .. } = term {
                    roll = true;
                }
            }

            if !roll {
                let roll = Term::Dice { n: 3, s: 6, t: None };
                let t = [&[roll], t].concat();
                return Rolls::new(&t);
            }

            for _ in 1..=repeat {
                let terms = t.iter().map(Term::with_value).collect::<Vec<_>>();

                let mut total = 0;
                let mut op: Term = Term::Add;
                let mut versus = None;

                for (term, mut values) in terms.clone() {
                    match term {
                        Term::Dice { .. } | Term::Num(_) => {
                            let i = if let Term::Dice { t, .. } = term {
                                if let Some(t) = t {
                                    values.sort_unstable();

                                    if t.is_positive() {
                                        values.iter().rev().take(t.abs() as usize).sum()
                                    } else if t.is_negative() {
                                        values.iter().take(t.abs() as usize).sum()
                                    } else {
                                        0
                                    }
                                } else {
                                    values.iter().sum()
                                }
                            } else {
                                values[0]
                            };

                            match op {
                                Term::Add => total += i,
                                Term::Sub => total -= i,
                                Term::Mul => total *= i,
                                Term::Div => total /= i,
                                Term::Rem => total %= i,
                                Term::Pow => total = total.pow(i as u32),
                                _ => unreachable!(),
                            }
                        }

                        Term::Add | Term::Sub | Term::Mul | Term::Div | Term::Rem | Term::Pow => {
                            op = term
                        }

                        Term::Versus { s, t } => {
                            versus = Some((s, t))
                        }

                        Term::Repeat(_) => (),
                    }
                }

                rolls.push(Roll { terms, total, versus });
            }

            Rolls(rolls)
        }
    }

    impl Term {
        fn with_value(&self) -> (Term, Vec<isize>) {
            match self {
                Term::Dice { n, s, .. } => {
                    let die = Uniform::new_inclusive(1, *s as isize);
                    let mut rng = rand::thread_rng();
                    (self.clone(), die.sample_iter(&mut rng).take(*n).collect())
                }
                Term::Num(i) => (self.clone(), vec![*i]),
                _ => (self.clone(), Vec::new()),
            }
        }
    }

    impl Display for Roll {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            for (term, values) in &self.terms {
                match term {
                    Term::Add
                    | Term::Sub
                    | Term::Mul
                    | Term::Div
                    | Term::Rem
                    | Term::Pow
                    | Term::Num(_) => write!(f, "{} ", term)?,
                    Term::Dice { .. } => write!(f, "{}{:?} ", term, values)?,
                    Term::Repeat(_) | Term::Versus{..} => (),
                }
            }

            // TODO disambiguate (on dice?) and impl other game types.
            //      enum Game { GURPS, d20, d100, Storyteller } ?
            if let Some((target, tag)) = &self.versus {
                if let Some(tag) = tag {
                    write!(f, "(Total: {:>2} vs {} {}: ", self.total, tag, target)?;
                } else {
                    write!(f, "(Total: {:>2} vs {}: ", self.total, target)?;
                };

                let target = *target;

                // GURPS 4th Edition success roll.
                let margin = target - self.total; // Roll under.

                if self.total < 5 || (target > 14 && self.total < 6) || (target > 15 && self.total < 7) {
                    write!(f, "Critical Success, Margin of {})", margin)
                } else if self.total > 16 || margin <= -10 {
                    if target > 15 && self.total == 17 {
                        write!(f, "Automatic Failure, Margin of {})", margin)
                    } else {
                        write!(f, "Critical Failure, Margin of {})", margin.abs())
                    }
                } else if margin < 0 {
                    write!(f, "Failure by {})", margin.abs())
                } else {
                    write!(f, "Success by {})", margin)
                }
            } else {
                write!(f, "(Total: {})", self.total)
            }
        }
    }

    impl Display for Rolls {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            let mut iter = self.0.iter();
            let first = match iter.next() {
                Some(first) => first,
                None => return Ok(()),
            };

            write!(f, "{}", first)?;

            for roll in iter {
                write!(f, "\n{}", roll)?;
            }

            Ok(())
        }
    }

    impl Display for Term {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            match self {
                Term::Dice { n, s, t } => {
                    if let Some(t) = t {
                        if t.is_positive() {
                            write!(f, "{}d{}b{}", n, s, t.abs())
                        } else if t.is_negative() {
                            write!(f, "{}d{}w{}", n, s, t.abs())
                        } else {
                            write!(f, "{}d{}x0", n, s)
                        }
                    } else {
                        write!(f, "{}d{}", n, s)
                    }
                }
                Term::Num(i) => write!(f, "{}", i),
                Term::Add => write!(f, "+"),
                Term::Sub => write!(f, "-"),
                Term::Mul => write!(f, "×"),
                Term::Div => write!(f, "/"),
                Term::Rem => write!(f, "%"),
                Term::Pow => write!(f, "^"),
                Term::Repeat(_) | Term::Versus{..} => Ok(()),
            }
        }
    }

    impl FromStr for Rolls {
        type Err = ParseRollError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"(?xi)
                    \d+ d (?:\d+)? (?: [bw] \d+ )?                  | # Term::Dice
                    [-+×x*/\\÷%^]                                   | # Term::<Op>
                    v(?:s|ersus)? \s+ (?:\D+?.*?-?)?? \s* -? \d+    | # Term::Vs
                    r(?:e|epeat)? \s* \d+                           | # Term::Repeat
                    -? \d+                                          # Term::Num")
                .unwrap();
            }

            let terms = RE.find_iter(&s)
                .map(|m| Term::from_str(&s[m.start()..m.end()]))
                .collect::<Result<Vec<Term>, _>>()?;

            Ok(Rolls::new(&terms))
        }
    }

    impl FromStr for Term {
        type Err = ParseRollError;

        fn from_str(input: &str) -> Result<Self, Self::Err> {
            let s = normalize_str(input);

            if s.starts_with('v') {
                lazy_static! { static ref RE: Regex = Regex::new(r"(?xi)
                    v(?:s|ersus)? \s+ (?: (?P<tag>\D+?.*?) -?)?? \s* (?P<target>-?\d+)
                ").unwrap(); }

                if let Some(cap) = RE.captures(input) {
                    let target = cap.name("target").unwrap().as_str().parse::<isize>()?;
                    let tag = cap.name("tag").map(|m| m.as_str().to_string());
                    Ok(Term::Versus { s: target, t: tag })
                } else {
                    Err(ParseRollError::Empty)
                }
            } else if s.starts_with('r') {
                let r = s.chars().skip_while(|c| !c.is_digit(10)).collect::<String>();
                Ok(Term::Repeat(r.parse()?))
            } else if s.contains('d') {
                let d: Vec<&str> = s.split(|c| c == 'd' || c == 'b' || c == 'w').collect();
                if s.contains('b') && d.len() == 3 {
                    Ok(Term::Dice {
                        n: d[0].parse()?,
                        s: d[1].parse()?,
                        t: Some(d[2].parse()?),
                    })
                } else if s.contains('w') && d.len() == 3 {
                    Ok(Term::Dice {
                        n: d[0].parse()?,
                        s: d[1].parse()?,
                        t: Some(-d[2].parse()?),
                    })
                } else {
                    Ok(Term::Dice {
                        n: d[0].parse()?,
                        s: d[1].parse().unwrap_or(6),
                        t: None,
                    })
                }
            } else {
                let mut c = s.chars();
                match c.next() {
                    Some('+') => Ok(Term::Add),
                    Some('-') => Ok(Term::Sub),
                    Some('*') => Ok(Term::Mul),
                    Some('x') => Ok(Term::Mul),
                    Some('×') => Ok(Term::Mul),
                    Some('/') => Ok(Term::Div),
                    Some('\\') => Ok(Term::Div),
                    Some('÷') => Ok(Term::Div),
                    Some('%') => Ok(Term::Rem),
                    Some('^') => Ok(Term::Pow),
                    Some(_) => Ok(Term::Num(s.parse()?)),
                    None => unreachable!(),
                }
            }
        }
    }

    #[derive(Clone, Debug)]
    pub enum ParseRollError {
        Int(ParseIntError),
        Empty,
    }


    impl Display for ParseRollError {
        fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
            match *self {
                ParseRollError::Int(ref err) => write!(f, "{}", err),
                ParseRollError::Empty => write!(f, "cannot parse term from empty string"),
            }
        }
    }

    impl Error for ParseRollError {
        fn source(&self) -> Option<&(dyn Error + 'static)> {
            match self {
                ParseRollError::Int(err) => Some(err),
                ParseRollError::Empty => None,
            }
        }
    }

    impl From<ParseIntError> for ParseRollError {
        fn from(err: ParseIntError) -> ParseRollError {
            ParseRollError::Int(err)
        }
    }


    fn normalize_str(s: &str) -> String {
        s.to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>()
    }
}
