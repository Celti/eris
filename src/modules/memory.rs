use crate::db::model::{Definition, Keyword};
use crate::db::Memory as DB;

use chrono::{TimeZone, Utc};
use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error as QueryError;
use diesel::result::Error::{DatabaseError, NotFound};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serenity::client::Context;
use serenity::framework::standard::macros::{command, group};
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, UserId};
use serenity::model::misc::Mentionable;
use serenity::prelude::*;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};

struct MemoryCache;
impl TypeMapKey for MemoryCache {
    type Value = std::collections::HashMap<ChannelId, Memory>;
}

struct Memory {
    idx: usize,
    def: Vec<Definition>,
}

impl Memory {
    pub fn send_to_channel(&self, ctx: &Context, channel_id: ChannelId) -> Result<Message, SerenityError> {
        channel_id.send_message(&ctx, |msg| {
            if self.def[self.idx].embedded {
                msg.embed(|e| e.image(&self.def[self.idx].definition))
            } else {
                msg.content(&self.def[self.idx].definition)
            }
        })
    }
}

#[derive(Debug)]
enum MemoryError {
    Denied,
    Exists,
    Invalid(String),
    NotFound,
    Query(QueryError),
}

impl Display for MemoryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            MemoryError::Denied => write!(f, "Permission denied."),
            MemoryError::Exists => write!(f, "Keyword or definition already exists."),
            MemoryError::Invalid(s) => write!(f, "Invalid argument: {}", s),
            MemoryError::NotFound => write!(f, "Keyword or definition not found."),
            MemoryError::Query(q) => write!(f, "{}", q),
        }
    }
}

impl From<QueryError> for MemoryError {
    fn from(err: QueryError) -> Self {
        MemoryError::Query(err)
    }
}

impl StdError for MemoryError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            MemoryError::Query(q) => Some(q),
            _ => None,
        }
    }
}

#[command]
#[description("Retrieve metadata for the current keyword.")]
#[num_args(0)]
fn details(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Some(memory) = ctx.data.read().get::<MemoryCache>().and_then(|m| m.get(&msg.channel_id)) {
        let kw   = &memory.def[memory.idx].keyword;
        let idx  = memory.idx + 1;
        let max  = memory.def.len();
        let user = UserId(memory.def[memory.idx].submitter as u64).mention();
        let ts   = memory.def[memory.idx].timestamp;

        say!(ctx, msg, "{} ({}/{}) submitted by {} at {}.", kw, idx, max, user, ts);
    }

    Ok(())
}

#[command]
#[description("Add a new embedded definition for a keyword.")]
#[min_args(2)]
fn embed(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    add_entry(&ctx, &msg, &mut args, true)
}

#[command]
#[description("Add a new definition for a keyword.")]
#[min_args(2)]
fn remember(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    add_entry(&ctx, &msg, &mut args, false)
}


#[command]
#[description("Find a keyword by partial match.")]
#[num_args(1)]
fn find(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let partial = args.single::<String>()?;

    match DB::find_keywords(&partial) {
        Ok(ref kw) if kw.is_empty() => say!(ctx, msg, "Sorry, I didn't find any keywords matching `{}`.", partial),
        Err(NotFound) => say!(ctx, msg, "Sorry, I didn't find any keywords matching `{}`.", partial),
        Ok(keywords) => say!(ctx, msg, "I found the following keywords: `{:?}`", keywords),
        Err(error) => Err(error)?,
    }

    Ok(())
}

#[command]
#[description("Remove a definition from a keyword.")]
#[min_args(2)]
fn forget(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let keyword = args.single::<String>()?;
    let definition = args.rest().to_string();
    let author: i64 = msg.author.id.into();

    let result = || -> Result<(), MemoryError> {
        let kw = match DB::get_keyword(&keyword) {
            Err(NotFound) => Err(MemoryError::NotFound)?,
            Err(error)    => Err(MemoryError::Query(error))?,
            Ok(keyword)   => keyword,
        };

        if (kw.hidden || kw.protect) && kw.owner != author {
            Err(MemoryError::Denied)?;
        }

        let def = match DB::get_definition(&keyword, &definition) {
            Err(NotFound) => Err(MemoryError::NotFound)?,
            Err(error) => Err(MemoryError::Query(error))?,
            Ok(definition) => definition,
        };

        DB::del_definition(&def)?;

        Ok(())
    }();

    match result {
        Err(MemoryError::Denied)       => say!(ctx, msg, "Sorry, you're not allowed to edit `{}`.", keyword),
        Err(MemoryError::Exists)       => unreachable!(),
        Err(MemoryError::Invalid(_))   => unreachable!(),
        Err(MemoryError::NotFound)     => say!(ctx, msg, "Sorry, I don't know that about `{}`.", keyword),
        Err(MemoryError::Query(error)) => Err(error)?,
        Ok(()) => say!(ctx, msg, "Entry removed for {}.", keyword),
    }

    Ok(())
}

#[command]
#[description("Find a keyword's definition by partial match.")]
#[min_args(2)]
fn search(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let keyword = args.single::<String>()?;
    let partial = args.rest().to_string();
    let author: i64 = msg.author.id.into();

    let result = || -> Result<Memory, MemoryError> {
        let kw = match DB::get_keyword(&keyword) {
            Err(NotFound) => Err(MemoryError::NotFound)?,
            Err(error)    => Err(MemoryError::Query(error))?,
            Ok(kw)   => kw,
        };

        if kw.hidden && kw.owner != author {
            Err(MemoryError::Denied)?;
        }

        let mut definitions = DB::find_definitions(&kw, &partial)?;

        if definitions.is_empty() {
            Err(MemoryError::NotFound)?
        };

        if kw.shuffle {
            definitions.shuffle(&mut thread_rng())
        };

        Ok(Memory { idx: 0, def: definitions })
    }();

    match result {
        Err(MemoryError::Denied)       => say!(ctx, msg, "Sorry, you're not allowed to view `{}`.", keyword),
        Err(MemoryError::Exists)       => unreachable!(),
        Err(MemoryError::Invalid(_))   => unreachable!(),
        Err(MemoryError::NotFound)     => say!(ctx, msg, "Sorry, I don't know anything about `{}` matching `{}`.", keyword, partial),
        Err(MemoryError::Query(error)) => Err(error)?,
        Ok(memory) => {
            memory.send_to_channel(&ctx, msg.channel_id)?;
            ctx.data.write().entry::<MemoryCache>().or_insert(Default::default()).insert(msg.channel_id, memory);
        }
    }

    Ok(())
}

#[command]
#[description("Retrieve the next definition for the current keyword.")]
#[num_args(0)]
fn next(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Some(memory) = ctx.data.write().get_mut::<MemoryCache>().and_then(|m| m.get_mut(&msg.channel_id)) {
        if memory.idx == memory.def.len() - 1 {
            memory.idx = 0;
        } else {
            memory.idx += 1;
        }

        memory.send_to_channel(&ctx, msg.channel_id)?;
    }

    Ok(())
}

#[command]
#[description("Retrieve the previous definition for the current keyword.")]
#[num_args(0)]
fn prev(ctx: &mut Context, msg: &Message) -> CommandResult {
    if let Some(memory) = ctx.data.write().get_mut::<MemoryCache>().and_then(|m| m.get_mut(&msg.channel_id)) {
        if memory.idx == 0 {
            memory.idx = memory.def.len() - 1;
        } else {
            memory.idx -= 1;
        }

        memory.send_to_channel(&ctx, msg.channel_id)?;
    }

    Ok(())
}

#[command]
#[description("Retrieve the definitions for a keyword.")]
#[num_args(1)]
fn recall(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let keyword = args.single::<String>()?;
    let author: i64 = msg.author.id.into();

    let result = || -> Result<Memory, MemoryError> {
        let kw = match DB::get_keyword(&keyword) {
            Err(NotFound) => Err(MemoryError::NotFound)?,
            Err(error)    => Err(MemoryError::Query(error))?,
            Ok(keyword)   => keyword,
        };

        if kw.hidden && kw.owner != author {
            Err(MemoryError::Denied)?;
        }

        let mut definitions = DB::get_definitions(&kw)?;

        if definitions.is_empty() {
            Err(MemoryError::NotFound)?
        };

        if kw.shuffle {
            definitions.shuffle(&mut thread_rng())
        };

        Ok(Memory { idx: 0, def: definitions })
    }();

    match result {
        Err(MemoryError::Denied)       => say!(ctx, msg, "Sorry, you're not allowed to view `{}`.", keyword),
        Err(MemoryError::Exists)       => unreachable!(),
        Err(MemoryError::Invalid(_))   => unreachable!(),
        Err(MemoryError::NotFound)     => say!(ctx, msg, "Sorry, I don't know anything about `{}`.", keyword),
        Err(MemoryError::Query(error)) => Err(error)?,
        Ok(memory) => {
            memory.send_to_channel(&ctx, msg.channel_id)?;
            ctx.data.write().entry::<MemoryCache>().or_insert(Default::default()).insert(msg.channel_id, memory);
        }
    }

    Ok(())
}

#[command]
#[description("Set the options for a keyword.")]
#[min_args(2)]
fn set(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let keyword = args.single::<String>()?;
    let user    = msg.author.id.into();

    let result = || -> Result<(), MemoryError> {
        let mut kw = match DB::get_keyword(&keyword) {
            Err(NotFound) => Err(MemoryError::NotFound)?,
            Err(error)    => Err(MemoryError::Query(error))?,
            Ok(keyword)   => keyword,
        };

        if (kw.hidden || kw.protect) && kw.owner != user {
            Err(MemoryError::Denied)?;
        }

        for option in args.iter::<String>() {
            match option.as_ref().unwrap().trim() {
                "bareword" => kw.bareword = !kw.bareword,
                "hidden"   => kw.hidden   = !kw.hidden,
                "protect"  => kw.protect  = !kw.protect,
                "shuffle"  => kw.shuffle  = !kw.shuffle,
                _          => Err(MemoryError::Invalid(option.unwrap()))?,
            };
        }

        kw.owner = user;

        DB::update_keyword(&kw)?;

        Ok(())
    }();

    match result {
        Err(MemoryError::Denied)       => say!(ctx, msg, "Sorry, you're not allowed to edit `{}`.", keyword),
        Err(MemoryError::Exists)       => unreachable!(),
        Err(MemoryError::Invalid(opt)) => say!(ctx, msg, "Sorry, I don't recognize the option `{}`.", opt),
        Err(MemoryError::NotFound)     => say!(ctx, msg, "Sorry, I don't know anything about `{}`.", keyword),
        Err(MemoryError::Query(error)) => Err(error)?,
        Ok(()) => say!(ctx, msg, "Options changed for {}.", keyword),
    }

    Ok(())
}

fn add_entry(ctx: &Context, msg: &Message, args: &mut Args, embedded: bool) -> CommandResult {
    let keyword = args.single::<String>()?;
    let definition = args.rest().to_string();

    let result = || -> Result<(), MemoryError> {
        let submitter = msg.author.id.into();
        let timestamp = Utc.from_utc_datetime(&msg.timestamp.naive_utc());
        let bareword = false;
        let hidden = false;
        let protect = false;
        let shuffle = true;

        let kw = Keyword {
            keyword: keyword.clone(),
            owner: submitter,
            bareword,
            hidden,
            protect,
            shuffle,
        };
        let def = Definition {
            keyword: keyword.clone(),
            definition,
            submitter,
            timestamp,
            embedded,
        };

        let kw = match DB::get_keyword(&keyword) {
            Err(NotFound) => DB::add_keyword(&kw)?,
            Err(error) => Err(MemoryError::Query(error))?,
            Ok(keyword) => keyword,
        };

        if (kw.hidden || kw.protect) && kw.owner != submitter {
            Err(MemoryError::Denied)?;
        }

        match DB::add_definition(&def) {
            Err(DatabaseError(UniqueViolation, _)) => Err(MemoryError::Exists)?,
            Err(error) => Err(MemoryError::Query(error))?,
            Ok(_) => (),
        };

        Ok(())
    }();

    match result {
        Err(MemoryError::Denied) => say!(ctx, msg, "Sorry, you're not allowed to edit `{}`.", keyword),
        Err(MemoryError::Exists) => say!(ctx, msg, "Sorry, I already know that about `{}`.", keyword),
        Err(MemoryError::Invalid(_)) => unreachable!(),
        Err(MemoryError::NotFound) => unreachable!(),
        Err(MemoryError::Query(error)) => Err(error)?,
        Ok(()) => say!(ctx, msg, "Entry added for {}.", keyword),
    }

    Ok(())
}

group!({
    name: "memory",
    options: {},
    commands: [details, embed, find, forget, search, next, prev, recall, remember, set]
});
