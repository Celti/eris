// FIXME use_extern_macros
// use serenity::command;

use crate::schema::*;
use crate::types::*;
use diesel::prelude::*;
use rand::Rng;
use diesel::result::Error::{DatabaseError as QueryViolation, NotFound as QueryNotFound};
use diesel::result::DatabaseErrorKind::UniqueViolation as Unique;

// match <kw> <part..> - load definitions matching <part> into cache
// find <partial> - list keywords matching <partial>

fn named_keyword_readable(name: &str, user: i64) -> keywords::BoxedQuery<DB> {
    keywords::table.find(name)
        .filter(keywords::owner.eq(user).or(keywords::hidden.eq(false)))
        .into_boxed()
}

fn named_keyword_writable(name: &str, user: i64) -> keywords::BoxedQuery<DB> {
    keywords::table.find(name)
        .filter(keywords::owner.eq(user).or(keywords::hidden.eq(false).and(keywords::protect.eq(false))))
        .into_boxed()
}

command!(recall(ctx, msg, args) {
    let mut map = ctx.data.lock();
    let find = args.single::<String>()?;

    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get()?;

    let result: QueryResult<_> = try {
        let keyword = named_keyword_readable(&find, msg.author.id.0 as i64).first::<KeywordEntry>(&*db)?;

        let mut entries = definitions::table.filter(definitions::keyword.eq(&find)).load::<DefinitionEntry>(&*db)?;

        if keyword.shuffle {
            rand::thread_rng().shuffle(&mut entries);
        }

        if entries.is_empty() {
            Err(QueryNotFound)?;
        }

        CurrentMemory { idx: 0, key: keyword, def: entries }
    };

    match result {
        Ok(ref c)          => msg.channel_id.send_message(|_| c.content())?,
        Err(QueryNotFound) => msg.channel_id.say(&format!("Sorry, I don't know anything about {}.", find))?,
        Err(_)             => msg.channel_id.say("Sorry, an error occurred.")?,
    };

    let mut cache = map.entry::<MemoryCache>().or_insert(std::collections::HashMap::new());
    cache.insert(msg.channel_id, result?);
});

command!(next(ctx, msg) {
    let mut map = ctx.data.lock();

    if let Some(cur) = map.get_mut::<MemoryCache>().and_then(|m| m.get_mut(&msg.channel_id)) {
        cur.next();
        msg.channel_id.send_message(|_| cur.content())?;
    } else {
        msg.channel_id.say("Next *what?*")?;
    }
});

command!(prev(ctx, msg) {
    let mut map = ctx.data.lock();

    if let Some(cur) = map.get_mut::<MemoryCache>().and_then(|m| m.get_mut(&msg.channel_id)) {
        cur.prev();
        msg.channel_id.send_message(|_| cur.content())?;
    } else {
        msg.channel_id.say("Previous *what?*")?;
    }
});

command!(details(ctx, msg) {
    let map = ctx.data.lock();

    if let Some(cur) = map.get::<MemoryCache>().and_then(|m| m.get(&msg.channel_id)) {
        msg.channel_id.say(&cur.details())?;
    } else {
        msg.channel_id.say("No.")?;
    }
});

fn add_entry(keyword: &str, definition: &str, submitter: i64, embed: bool, db: &DbConnection) -> QueryResult<usize> {
    if named_keyword_writable(keyword, submitter).first::<KeywordEntry>(&**db).optional()?.is_none() {
        diesel::insert_into(keywords::table)
            .values(keywords::keyword.eq(keyword))
            .execute(&**db).or(Err(QueryNotFound))?;
    }

    Ok(diesel::insert_into(definitions::table)
        .values(&NewDefinitionEntry {keyword, definition, submitter, embed})
        .execute(&**db)?)
}

command!(remember_embed(ctx, msg, args) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get()?;

    let keyword    = args.single::<String>()?;
    let definition = args.multiple::<String>()?.join(" ");

    match add_entry(&keyword, &definition, msg.author.id.0 as i64, true, &db) {
        Ok(1) => msg.channel_id.say(&format!("Entry added for {}.", keyword))?,
        Err(QueryViolation(Unique,_)) => msg.channel_id.say(&format!("I already know that about {}.", keyword))?,
        Err(QueryNotFound) => msg.channel_id.say(&format!("You're not cleared to edit {}.", keyword))?,
        Ok(_) | Err(_) => msg.channel_id.say("Sorry, an error occurred.")?,
    };
});

command!(remember(ctx, msg, args) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get()?;

    let keyword    = args.single::<String>()?;
    let definition = args.multiple::<String>()?.join(" ");

    match add_entry(&keyword, &definition, msg.author.id.0 as i64, false, &db) {
        Ok(1) => msg.channel_id.say(&format!("Entry added for {}.", keyword))?,
        Err(QueryViolation(Unique,_)) => msg.channel_id.say(&format!("I already know that about {}.", keyword))?,
        Err(QueryNotFound) => msg.channel_id.say(&format!("You're not cleared to edit {}.", keyword))?,
        Ok(_) | Err(_) => msg.channel_id.say("Sorry, an error occurred.")?,
    };
});

command!(forget(ctx, msg, args) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get()?;

    let keyword    = args.single::<String>()?;
    let definition = args.multiple::<String>()?.join(" ");

    let result: QueryResult<_> = try {
        named_keyword_writable(&keyword, msg.author.id.0 as i64).first::<KeywordEntry>(&*db)?;
        diesel::delete(definitions::table.find((&keyword, &definition))).execute(&*db)?
    };

    match result {
        Ok(1) => msg.channel_id.say(&format!("Entry removed for {}.", keyword))?,
        Ok(0) => msg.channel_id.say(&format!("Could not remove entry for {}.", keyword))?,
        Err(QueryNotFound) => msg.channel_id.say(&format!("You're not cleared to edit {}.", keyword))?,
        Ok(_) | Err(_) => msg.channel_id.say("Sorry, an error occurred.")?,
    };
});

command!(set(ctx, msg, args) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get()?;

    let mut update = NewKeywordEntry::default();

    let keyword = args.single::<String>()?;
    let option  = args.single::<String>()?;
    let value   = args.single::<bool>()?;
    let owner   = msg.author.id.0 as i64;

    match option.trim() {
        "bare"    => update.bare(value, owner),
        "hidden"  => update.hidden(value, owner),
        "protect" => update.protect(value, owner),
        "shuffle" => update.shuffle(value),
        _         => Err("invalid option type")?,
    };

    let result: QueryResult<_> = try {
        named_keyword_writable(&keyword, msg.author.id.0 as i64).first::<KeywordEntry>(&*db)?;
        diesel::update(keywords::table.find(&keyword)).set(&update).execute(&*db)?
    };

    match result {
        Ok(1) => msg.channel_id.say(&format!("Option `{}` for {} now set to {}", option, keyword, value))?,
        Ok(0) => msg.channel_id.say(&format!("Could not set `{}` for {}.", option, keyword))?,
        Err(QueryNotFound) => msg.channel_id.say(&format!("You're not cleared to edit {}.", keyword))?,
        Ok(_) | Err(_) => msg.channel_id.say("Sorry, an error occurred.")?,
    };
});
