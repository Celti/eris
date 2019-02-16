use crate::db::DB;
use crate::db::model::{Definition, Keyword};

use chrono::{TimeZone, Utc};
use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error as QueryError;
use failure::Fail;
use rand::seq::SliceRandom;
use rand::thread_rng;
use self::QueryError::{NotFound, DatabaseError};
use serenity::builder::CreateMessage;
use serenity::framework::standard::{Args, CommandError};
use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, UserId};
use serenity::model::misc::Mentionable;
use serenity::prelude::TypeMapKey;

struct MemoryCache;
impl TypeMapKey for MemoryCache {
    type Value = std::collections::HashMap<ChannelId, Memory>;
}

struct Memory {
    idx: usize,
    def: Vec<Definition>,
}

impl Memory {
    pub fn to_message_content(&self) -> CreateMessage {
        if self.def[self.idx].embedded {
            CreateMessage::default().embed(|e| e.image(&self.def[self.idx].definition))
        } else {
            CreateMessage::default().content(&self.def[self.idx].definition)
        }
    }
}

#[derive(Debug, Fail)]
enum MemoryError {
    #[fail(display = "Permission denied.")]
    Denied,
    #[fail(display = "Definition exists.")]
    Exists,
    #[fail(display = "Invalid argument: {}", _0)]
    Invalid(String),
    #[fail(display = "Keyword not found.")]
    NotFound,
    #[fail(display = "{}", _0)]
    Other(#[cause] QueryError)
}

cmd!(Details(ctx, msg, _args)
     aliases: ["details"],
     desc: "Retrieve metadata for the current keyword definition.", num_args: 0, {
    if let Some(memory) = ctx.data.lock().get::<MemoryCache>().and_then(|m| m.get(&msg.channel_id)) {
        let kw   = &memory.def[memory.idx].keyword;
        let idx  = memory.idx + 1;
        let max  = memory.def.len();
        let user = UserId(memory.def[memory.idx].submitter as u64).mention();
        let ts   = memory.def[memory.idx].timestamp;

        msg.channel_id.say(&format!("{} ({}/{}) submitted by {} at {}.", kw, idx, max, user, ts))?;
    }
});

cmd!(Embed(_ctx, msg, args)
     aliases: ["embed"],
     desc: "Add a new keyword embed.", min_args: 2, {
    add_entry(&msg, &mut args, true)?;
});

cmd!(Find(_ctx, msg, args)
     aliases: ["find"],
     desc: "Find a keyword by match.", num_args: 1, {
    let partial = args.single::<String>()?;

    match DB.find_keywords(&partial) {
        Ok(ref kw) if kw.is_empty() => say!(msg.channel_id, "Sorry, I didn't find any keywords matching `{}`.", partial),
        Err(NotFound) => say!(msg.channel_id, "Sorry, I didn't find any keywords matching `{}`.", partial),
        Ok(keywords) => say!(msg.channel_id, "I found the following keywords: `{:?}`", keywords),
        Err(error) => Err(error)?,
    };
});

cmd!(Forget(_ctx, msg, args)
     aliases: ["forget"],
     desc: "Forget a specific keyword definition.", min_args: 2, {
    let keyword = args.single::<String>()?;
    let definition = args.rest().to_string();

    let result: Result<(), MemoryError> = try {
        let kw = match DB.get_keyword(&keyword) {
            Err(NotFound) => Err(MemoryError::NotFound)?,
            Err(error)    => Err(MemoryError::Other(error))?,
            Ok(keyword)   => keyword,
        };

        if (kw.hidden || kw.protect) && kw.owner != msg.author.id.into():i64 {
            Err(MemoryError::Denied)?;
        }

        let def = match DB.get_definition(&keyword, &definition) {
            Err(NotFound) => Err(MemoryError::NotFound)?,
            Err(error) => Err(MemoryError::Other(error))?,
            Ok(definition) => definition,
        };

        DB.del_definition(&def).map_err(MemoryError::Other)?;
    };

    match result {
        Err(MemoryError::Denied)       => { say!(msg.channel_id, "Sorry, you're not allowed to edit `{}`.", keyword); }
        Err(MemoryError::Exists)       => { unreachable!() }
        Err(MemoryError::Invalid(_))   => { unreachable!() }
        Err(MemoryError::NotFound)     => { say!(msg.channel_id, "Sorry, I don't know that about `{}`.", keyword); }
        Err(MemoryError::Other(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "Entry removed for {}.", keyword); }
    }
});

cmd!(Match(ctx, msg, args)
     aliases: ["match"],
     desc: "Match against a keyword's definitions.", min_args: 2, {
    let keyword = args.single::<String>()?;
    let partial = args.rest().to_string();

    let result: Result<Memory, MemoryError> = try {
        let kw = match DB.get_keyword(&keyword) {
            Err(NotFound) => Err(MemoryError::NotFound)?,
            Err(error)    => Err(MemoryError::Other(error))?,
            Ok(kw)   => kw,
        };

        if kw.hidden && kw.owner != msg.author.id.into():i64 {
            Err(MemoryError::Denied)?;
        }

        let mut definitions = DB.find_definitions(&kw, &partial).map_err(MemoryError::Other)?;

        if definitions.is_empty() {
            Err(MemoryError::NotFound)?;
        }

        if kw.shuffle {
            definitions.shuffle(&mut thread_rng());
        }

        Memory { idx: 0, def: definitions }
    };

    match result {
        Err(MemoryError::Denied)       => { say!(msg.channel_id, "Sorry, you're not allowed to view `{}`.", keyword); }
        Err(MemoryError::Exists)       => { unreachable!() }
        Err(MemoryError::Invalid(_))   => { unreachable!() }
        Err(MemoryError::NotFound)     => {
            say!(msg.channel_id, "Sorry, I don't know anything about `{}` matching `{}`.", keyword, partial);
        }
        Err(MemoryError::Other(error)) => { Err(error)?; }
        Ok(memory) => {
            msg.channel_id.send_message(|_| memory.to_message_content())?;
            ctx.data.lock().entry::<MemoryCache>().or_insert(Default::default()).insert(msg.channel_id, memory);
        }
    }
});

cmd!(Next(ctx, msg, _args)
     aliases: ["next"],
     desc: "Retrieve the current keyword's next definition.", num_args: 0, {
    if let Some(memory) = ctx.data.lock().get_mut::<MemoryCache>().and_then(|m| m.get_mut(&msg.channel_id)) {
        if memory.idx == memory.def.len() - 1 {
            memory.idx = 0;
        } else {
            memory.idx += 1;
        }

        msg.channel_id.send_message(|_| memory.to_message_content())?;
    }
});

cmd!(Prev(ctx, msg, _args)
     aliases: ["prev"],
     desc: "Retrieve the current keyword's previous definition.", num_args: 0, {
    if let Some(memory) = ctx.data.lock().get_mut::<MemoryCache>().and_then(|m| m.get_mut(&msg.channel_id)) {
        if memory.idx == 0 {
            memory.idx = memory.def.len() - 1;
        } else {
            memory.idx -= 1;
        }

        msg.channel_id.send_message(|_| memory.to_message_content())?;
    }
});

cmd!(Recall(ctx, msg, args)
     aliases: ["recall"],
     desc: "Retrieve a keyword's definitions.", num_args: 1, {
    let keyword = args.single::<String>()?;

    let result: Result<Memory, MemoryError> = try {
        let kw = match DB.get_keyword(&keyword) {
            Err(NotFound) => Err(MemoryError::NotFound)?,
            Err(error)    => Err(MemoryError::Other(error))?,
            Ok(keyword)   => keyword,
        };

        if kw.hidden && kw.owner != msg.author.id.into():i64 {
            Err(MemoryError::Denied)?;
        }

        let mut definitions = DB.get_definitions(&kw).map_err(MemoryError::Other)?;

        if kw.shuffle {
            definitions.shuffle(&mut thread_rng());
        }

        if definitions.is_empty() {
            Err(MemoryError::NotFound)?;
        }

        Memory { idx: 0, def: definitions }
    };

    match result {
        Err(MemoryError::Denied)       => { say!(msg.channel_id, "Sorry, you're not allowed to view `{}`.", keyword); }
        Err(MemoryError::Exists)       => { unreachable!() }
        Err(MemoryError::Invalid(_))   => { unreachable!() }
        Err(MemoryError::NotFound)     => { say!(msg.channel_id, "Sorry, I don't know anything about `{}`.", keyword); }
        Err(MemoryError::Other(error)) => { Err(error)?; }
        Ok(memory) => {
            msg.channel_id.send_message(|_| memory.to_message_content())?;
            ctx.data.lock().entry::<MemoryCache>().or_insert(Default::default()).insert(msg.channel_id, memory);
        }
    }
});

cmd!(Remember(_ctx, msg, args)
     aliases: ["remember"],
     desc: "Add a new keyword definition.", min_args: 2, {
    add_entry(&msg, &mut args, false)?;
});

cmd!(Set(_ctx, msg, args)
     aliases: ["set"],
     desc: "Set keyword options.", min_args: 2, {
    let keyword = args.single::<String>()?;
    let user    = msg.author.id.into():i64;

    let result: Result<(), MemoryError> = try {
        let mut kw = match DB.get_keyword(&keyword) {
            Err(NotFound) => Err(MemoryError::NotFound)?,
            Err(error)    => Err(MemoryError::Other(error))?,
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

        DB.update_keyword(&kw).map_err(MemoryError::Other)?;
    };

    match result {
        Err(MemoryError::Denied)       => { say!(msg.channel_id, "Sorry, you're not allowed to edit `{}`.", keyword); }
        Err(MemoryError::Exists)       => { unreachable!() }
        Err(MemoryError::Invalid(opt)) => { say!(msg.channel_id, "Sorry, I don't recognize the option `{}`.", opt); }
        Err(MemoryError::NotFound)     => { say!(msg.channel_id, "Sorry, I don't know anything about `{}`.", keyword); }
        Err(MemoryError::Other(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "Options changed for {}.", keyword); }
    }
});

fn add_entry(msg: &Message, args: &mut Args, embedded: bool) -> Result<(), CommandError> {
    let keyword    = args.single::<String>()?;
    let definition = args.rest().to_string();

    let result: Result<(), MemoryError> = try {
        let submitter  = msg.author.id.into():i64;
        let timestamp  = Utc.from_utc_datetime(&msg.timestamp.naive_utc());
        let bareword   = false;
        let hidden     = false;
        let protect    = false;
        let shuffle    = true;

        let kw  = Keyword { keyword: keyword.clone(), owner: submitter, bareword, hidden, protect, shuffle };
        let def = Definition { keyword: keyword.clone(), definition, submitter, timestamp, embedded };

        let kw = match DB.get_keyword(&keyword) {
            Err(NotFound) => DB.add_keyword(&kw).map_err(MemoryError::Other)?,
            Err(error)    => Err(MemoryError::Other(error))?,
            Ok(keyword)   => keyword,
        };

        if (kw.hidden || kw.protect) && kw.owner != submitter {
            Err(MemoryError::Denied)?;
        }

        match DB.add_definition(&def) {
            Err(DatabaseError(UniqueViolation,_)) => Err(MemoryError::Exists)?,
            Err(error) => Err(MemoryError::Other(error))?,
            Ok(_) => (),
        };
    };

    match result {
        Err(MemoryError::Denied)       => { say!(msg.channel_id, "Sorry, you're not allowed to edit `{}`.", keyword); }
        Err(MemoryError::Exists)       => { say!(msg.channel_id, "Sorry, I already know that about `{}`.", keyword); }
        Err(MemoryError::Invalid(_))   => { unreachable!() }
        Err(MemoryError::NotFound)     => { unreachable!() }
        Err(MemoryError::Other(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "Entry added for {}.", keyword); }
    }

    Ok(())
}

grp![Details, Embed, Find, Forget, Match, Next, Prev, Recall, Remember, Set];
