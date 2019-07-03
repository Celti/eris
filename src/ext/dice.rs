use lazy_static::lazy_static;
use rand::distributions::{Distribution, Uniform};
use regex::Regex;
use std::fmt;
use std::str::FromStr;
use std::{error::Error, num::ParseIntError};

#[derive(Clone, Debug)]
pub struct DiceRoll { rolls: Vec<Roll> }

impl DiceRoll {
    fn new(mut t: Vec<Term>) -> Self {
        let repeat;
        let versus;

        if ! t.iter().any(Term::is_dice) {
            t.push(Term::Dice { n: 3, s: 6, t: None });
        }

        if let Some(Term::Repeat(i)) = t.iter().find(|t| t.is_repeat()) {
            repeat = *i;
        } else {
            repeat = 1;
        }

        if let Some(Term::Versus {s, t}) = t.iter().find(|t| t.is_versus()) {
            versus = Some((*s, t.clone()));
        } else {
            versus = None;
        }

        let terms = t.into_iter().filter(|t| ! t.is_repeat() && ! t.is_versus());
        let mut rolls = Vec::with_capacity(repeat);

        for _ in 1..=repeat {
            let terms: Vec<_> = terms.clone().map(Term::with_rolls).collect();
            let versus = versus.clone();

            let mut last = &Term::Add;
            let mut total = 0;

            for (term, rolls) in terms.iter() {
                match term {
                    Term::Dice { t, .. } => {
                        let mut rolls = rolls.clone().unwrap().0;
                        let sum = {
                            if let Some(t) = t {
                                rolls.sort_unstable();

                                if t.is_positive() {
                                    rolls.iter().rev().take(t.abs() as usize).sum()
                                } else if t.is_negative() {
                                    rolls.iter().take(t.abs() as usize).sum()
                                } else {
                                    0
                                }
                            } else {
                                rolls.iter().sum()
                            }
                        };

                        total = last.op(total, sum);
                    }

                    Term::Num(i) => total = last.op(total, *i),
                    _ => last = term,

                }
            }

            rolls.push(Roll { terms, total, versus });
        }

        DiceRoll { rolls }
    }
}

impl fmt::Display for DiceRoll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.rolls.iter();
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

impl FromStr for DiceRoll {
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

        Ok(DiceRoll::new(terms))
    }
}

#[derive(Clone, Debug)]
struct Roll {
    terms: Vec<(Term, Option<Rolls>)>,
    total: isize,
    versus: Option<(isize, Option<String>)>,
}

impl fmt::Display for Roll {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (term, rolls) in &self.terms {
            match (term, rolls) {
                (Term::Dice { .. }, Some(rolls)) => write!(f, "{}{} ", term, rolls)?,
                (Term::Repeat(_), _) | (Term::Versus{..}, _) => (),
                (_,_) => write!(f, "{} ", term)?,
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

#[derive(Clone, Debug)]
struct Rolls(Vec<isize>);

impl fmt::Display for Rolls {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut iter = self.0.iter();
        let first = match iter.next() {
            Some(first) => first,
            None => return Ok(()),
        };

        write!(f, "[{}", first)?;

        for roll in iter {
            write!(f, ", {}", roll)?;
        }

        write!(f, "]")
    }
}

#[derive(Clone, Debug)]
enum Term {
    Dice {
        n: usize,
        s: isize,
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

impl Term {
    fn is_dice(&self) -> bool {
        if let Term::Dice{..} = self {
            true
        } else {
            false
        }
    }

    fn is_repeat(&self) -> bool {
        if let Term::Repeat(_) = self {
            true
        } else {
            false
        }
    }

    fn is_versus(&self) -> bool {
        if let Term::Versus{..} = self {
            true
        } else {
            false
        }
    }

    fn op(&self, lhs: isize, rhs: isize) -> isize {
        match self {
            Term::Add => lhs + rhs,
            Term::Sub => lhs - rhs,
            Term::Mul => lhs * rhs,
            Term::Div => lhs / rhs,
            Term::Rem => lhs % rhs,
            Term::Pow => lhs.pow(rhs as u32),
            _ => panic!("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA"),
        }
    }

    fn with_rolls(self) -> (Term, Option<Rolls>) {
        match self {
            Term::Dice { n, s, .. } => {
                let die = Uniform::new_inclusive(1, s);
                let mut rng = rand::thread_rng();
                (self, Some(Rolls(die.sample_iter(&mut rng).take(n).collect())))
            }
            _ => (self, None),
        }
    }
}

impl fmt::Display for Term {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            Term::Repeat(_) | Term::Versus{..} => Ok(()),
            Term::Num(i) => write!(f, "{}", i),
            Term::Add => write!(f, "+"),
            Term::Sub => write!(f, "-"),
            Term::Mul => write!(f, "×"),
            Term::Div => write!(f, "/"),
            Term::Rem => write!(f, "%"),
            Term::Pow => write!(f, "^"),
        }
    }
}

impl FromStr for Term {
    type Err = ParseRollError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let s = input
            .to_lowercase()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect::<String>();

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

impl PartialEq for Term {
    fn eq(&self, other: &Term) -> bool {
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}


#[derive(Clone, Debug)]
pub enum ParseRollError {
    Int(ParseIntError),
    Empty,
}

impl fmt::Display for ParseRollError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
