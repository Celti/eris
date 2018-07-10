use serenity::command;
use self::parse::Roll;
use regex::Regex;
use lazy_static::{lazy_static, __lazy_static_create, __lazy_static_internal}; // FIXME use_extern_macros

// TODO versus modes for games besides GURPS

command!(roll(_ctx, msg, args) {
    let res = if let Ok(r) = args.single::<Roll>() { r }
        else { "3d6".parse().unwrap() };

    let rem = if let Ok(s) = args.multiple::<String>() { s.join(" ") }
        else { String::new() };

    let name = crate::util::cached_display_name(msg.channel_id, msg.author.id)?;

    let mut out = Vec::new();
    let mut comment = String::new();

    lazy_static! { static ref RE: Regex = Regex::new(r"(?x) (?:
        \s* v(?:s|ersus)? \s* (?P<t1>\S+?.*?)?? [\s-] (?P<v1>-?\d+) (?: \s* r(?:e|epeat)? \s* (?P<r2>\d+) \s* )? |
        \s* r(?:e|epeat)? \s* (?P<r1>\d+) (?: \s* v(?:s|ersus)? \s* (?P<t2>\S+?.*?)? [\s-] (?P<v2>-?\d+) \s* )?
    ) \s* (?:\#(?P<c>.+)$)?").unwrap(); }

    if let Some(caps) = RE.captures(&rem) {
        let rolls: Vec<Roll> = if let Some(repeat) = caps.name("r1").or_else(|| caps.name("r2")) {
            let x = repeat.as_str().parse::<usize>()?;
            res.into_iter().take(x).collect()
        } else {
            vec![res]
        };
    
        if let Some(c) = caps.name("c") {
            comment.push_str(&format!(" `{}`", c.as_str()));
        }

        if let Some(versus) = caps.name("v1").or_else(|| caps.name("v2")) {
            let target = versus.as_str().parse::<isize>()?;

            let skill = if let Some(tag) = caps.name("t1").or_else(|| caps.name("t2")) {
                format!("{} {}", tag.as_str(), target)
            } else {
                format!("{}", target)
            };

            for res in rolls {
                // GURPS 4th Edition success roll.
                let margin = target - res.total; // Roll under.

                if res.total < 5 || (target > 14 && res.total < 6) || (target > 15 && res.total < 7) {
                    out.push(format!("{:>2} vs {}: Success by {} (CRITICAL SUCCESS)", res.total, skill, margin));
                } else if res.total > 16 || margin <= -10 {
                    if target > 15 && res.total == 17 {
                        out.push(format!("{:>2} vs {}: Margin of {} (Automatic Failure)", res.total, skill, margin));
                    } else {
                        out.push(format!("{:>2} vs {}: Failure by {} (CRITICAL FAILURE)", res.total, skill, margin.abs()));
                    }
                } else if margin < 0 {
                    out.push(format!("{:>2} vs {}: Failure by {}", res.total, skill, margin.abs()));
                } else {
                    out.push(format!("{:>2} vs {}: Success by {}", res.total, skill, margin));
                }
            }
        } else {
            for res in rolls {
                out.push(res.to_string());
            }
        }
    } else {
        out.push(res.to_string());
    }


    let _sent = msg.channel_id.send_message(|m| {
        m.content(
            format!("**{} rolled:**{}\n```\n{}\n```", name, comment, out.join("\n"))
        ).reactions(vec!['🎲'])
    })?;

});

mod parse {
    use lazy_static::{lazy_static, __lazy_static_create, __lazy_static_internal}; // FIXME use_extern_macros
    use rand::distributions::{Distribution, Uniform};
    use regex::Regex;
    use std::{fmt::{Display, Formatter, Result as FmtResult}, str::FromStr};
    use std::{error::Error as StdError, num::ParseIntError};

    fn normalize_str(s: &str) -> String {
        s.to_lowercase()
         .chars()
         .filter(|c| !c.is_whitespace())
         .collect::<String>()
    }

    #[derive(Clone, Debug)]
    crate struct Roll {
        pub terms: Vec<(Term, Vec<isize>)>,
        pub total: isize,
    }

    impl Roll {
        pub fn new(t: Vec<Term>) -> Self {
            let terms = t.into_iter().map(|term| term.with_value()).collect::<Vec<_>>();

            let mut total = 0;
            let mut op: Term = Term::Add;

            for (term, values) in &mut terms.clone() {
                match term {
                    Term::Dice{..} | Term::Num(_) => {
                        let i = if let Term::Dice{t, ..} = term {
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
                            _         => unreachable!(),
                        }
                    }

                    Term::Add | Term::Sub | Term::Mul |
                    Term::Div | Term::Rem | Term::Pow => op = *term,
                }
            }

            Roll { terms, total }
        }
    }

    impl Display for Roll {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            let mut out = String::new();

            for (term, values) in &self.terms {
                out.push_str(" ");
                match term {
                    Term::Add | Term::Sub | Term::Mul |
                    Term::Div | Term::Rem | Term::Pow |
                    Term::Num(_) => out.push_str(&term.to_string()),
                    Term::Dice {..} => out.push_str(&format!("{}{:?}", term, values)),
                }
            }

            write!(f, "{} (Total: {})", out, self.total)
        }
    }

    impl FromStr for Roll {
        type Err = ParseRollError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            lazy_static! {
                static ref RE: Regex = Regex::new(r"(?x)
                    \d+ d \d+ (?: [bw] \d+ ) | # Term::Dice
                    [-+×x*/\\÷%^]            | # Term::<Op>
                    -? \d+                     # Term::Num
                ").unwrap();
            }

            let s = normalize_str(s);
            let mut terms = Vec::new();
            let matches = RE.find_iter(&s);

            for m in matches {
                terms.push(Term::from_str(&s[m.start()..m.end()])?);
            }

            if terms.is_empty() {
                Err(ParseRollError::Empty)
            } else {
                Ok(Roll::new(terms))
            }
        }
    }

    #[derive(Clone, Debug)]
    pub enum ParseRollError {
        Int(ParseIntError),
        Empty,
    }

    impl Display for ParseRollError {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            match *self {
                ParseRollError::Int(ref err) => write!(f, "{}", err),
                ParseRollError::Empty        => write!(f, "cannot parse term from empty string"),
            }
        }
    }

    impl StdError for ParseRollError {
        fn cause(&self) -> Option<&std::error::Error> {
            match *self {
                ParseRollError::Int(ref err) => Some(err),
                ParseRollError::Empty        => None,
            }
        }
    }

    impl From<ParseIntError> for ParseRollError {
        fn from(err: ParseIntError) -> ParseRollError {
            ParseRollError::Int(err)
        }
    }

    impl IntoIterator for Roll {
        type Item = Roll;
        type IntoIter = RollIterator;

        fn into_iter(self) -> Self::IntoIter {
            RollIterator {
                roll: self,
            }
        }
    }

    crate struct RollIterator {
        roll: Roll,
    }

    impl Iterator for RollIterator {
        type Item = Roll;

        fn next(&mut self) -> Option<Roll> {
            let (terms, _): (Vec<Term>, Vec<_>) = self.roll.terms.clone().into_iter().unzip();

            if terms.is_empty() {
                None
            } else {
                Some(Roll::new(terms))
            }
        }
    }


    #[derive(Clone, Copy, Debug)]
    crate enum Term {
        Dice { n: usize, s: usize, t: Option<isize> },
        Num(isize),
        Add,
        Sub,
        Mul,
        Div,
        Rem,
        Pow,
    }

    impl Term {
        fn with_value(self) -> (Term, Vec<isize>) {
            match self {
                Term::Dice{n, s, ..} => {
                    let die = Uniform::new_inclusive(1, s as isize);
                    let mut rng = rand::thread_rng();
                    (self, die.sample_iter(&mut rng).take(n).collect())
                }
                Term::Num(i) => (self, vec![i]),
                _            => (self, Vec::new()),
            }
        }
    }

    impl Display for Term {
        fn fmt(&self, f: &mut Formatter) -> FmtResult {
            match self {
                Term::Dice{n,s,t} => {
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
                Term::Num(i)    => write!(f, "{}", i),
                Term::Add       => write!(f, "+"),
                Term::Sub       => write!(f, "-"),
                Term::Mul       => write!(f, "×"),
                Term::Div       => write!(f, "/"),
                Term::Rem       => write!(f, "%"),
                Term::Pow       => write!(f, "^"),
            }
        }
    }

    impl FromStr for Term {
        type Err = ParseRollError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            let s = normalize_str(s);

            if s.contains('d') {
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
                        s: d[1].parse()?,
                        t: None,
                    })
                }
            } else {
                let mut c = s.chars();
                match c.next() {
                    Some('+')  => Ok(Term::Add),
                    Some('-')  => Ok(Term::Sub),
                    Some('*')  => Ok(Term::Mul),
                    Some('x')  => Ok(Term::Mul),
                    Some('×')  => Ok(Term::Mul),
                    Some('/')  => Ok(Term::Div),
                    Some('\\') => Ok(Term::Div),
                    Some('÷')  => Ok(Term::Div),
                    Some('%')  => Ok(Term::Rem),
                    Some('^')  => Ok(Term::Pow),
                    Some(_)    => Ok(Term::Num(s.parse()?)),
                    None       => unreachable!(),
                }
            }
        }
    }
}
