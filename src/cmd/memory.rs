use serenity::command;
use serenity::framework::standard::CommandError;
use serenity::prelude::Mentionable;
use serenity::model::prelude::Message;
use crate::types::*;
use chrono::Utc;
use rand::Rng;
use serde_rusqlite::{from_row, from_rows, to_params};

// match <kw> <part..> - load definitions matching <part> into cache
// find <partial> - list keywords matching <partial>

command!(recall(ctx, msg, args) {
    let mut map = ctx.data.lock();
    let find = args.single::<String>()?;

    let result: Result<_,CommandError> = do catch {
        let handle = map.get::<DatabaseHandle>().unwrap();
        let db     = handle.get()?;

        let keyword = check_keyword(&db, &find, &msg)?.unwrap_or_default();

        let mut stmt = db.prepare("SELECT * FROM definitions WHERE keyword=?")?;
        let mut rows = from_rows::<DefinitionEntry>(stmt.query(&[&find])?).collect::<Vec<_>>();

        if rows.is_empty() {
            Err("")?;
        }

        if keyword.shuffle {
            rand::thread_rng().shuffle(&mut rows);
        }

        CurrentMemory { idx: 0, key: keyword, def: rows }
    };

    if let Ok(cur) = result {
        msg.channel_id.say(&cur.content())?;
        let mut cache = map.entry::<MemoryCache>().or_insert(std::collections::HashMap::new());
        cache.insert(msg.channel_id, cur);
    } else {
        msg.channel_id.say(&format!("Sorry, I don't know anything about {}.", find))?;
        result?;
    }
});

command!(next(ctx, msg) {
    let mut map = ctx.data.lock();

    if let Some(cur) = map.get_mut::<MemoryCache>().and_then(|m| m.get_mut(&msg.channel_id)) {
        cur.next();
        msg.channel_id.say(&cur.content())?;
    } else {
        msg.channel_id.say("Next *what?*")?;
    }
});

command!(prev(ctx, msg) {
    let mut map = ctx.data.lock();

    if let Some(cur) = map.get_mut::<MemoryCache>().and_then(|m| m.get_mut(&msg.channel_id)) {
        cur.prev();
        msg.channel_id.say(&cur.content())?;
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

command!(remember(ctx, msg, args) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get()?;

    let keyword    = args.single::<String>()?;
    let definition = args.multiple::<String>()?.join(" ");
    let submitter  = msg.author.id;
    let timestamp  = msg.timestamp.with_timezone(&Utc);

    if check_keyword(&db, &keyword, &msg)?.is_none() {
        db.execute("INSERT OR IGNORE INTO keywords VALUES (?, ?, true, false, false, false)",
                   &to_params((&keyword, &submitter))?.to_slice() )?;
    }

    if let Ok(1) = db.execute("INSERT INTO definitions VALUES (?, ?, ?, ?)",
                              &to_params((&keyword, &definition, &submitter, &timestamp))?.to_slice()) {
        msg.channel_id.say(&format!("Entry added for {}.", keyword))?;
    } else {
        msg.channel_id.say(&format!("I already know that about {}.", keyword))?;
    }
});

command!(forget(ctx, msg, args) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get()?;

    let keyword    = args.single::<String>()?;
    let definition = args.multiple::<String>()?.join(" ");

    check_keyword(&db, &keyword, &msg)?;

    if let Ok(1) = db.execute("DELETE FROM definitions WHERE keyword = ? AND definition = ?", &[&keyword, &definition]) {
        msg.channel_id.say(&format!("Entry removed for {}.", keyword))?;
    } else {
        msg.channel_id.say(&format!("Could not remove entry for {}.", keyword))?;
    }
});

command!(set(ctx, msg, args) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get()?;

    let keyword = args.single::<String>()?;
    let option  = args.single::<String>()?;
    let value   = args.single::<bool>()?;

    let mut statement = match option.trim() {
        "protect" => db.prepare("UPDATE OR IGNORE keywords SET protect = ? WHERE keyword = ?")?,
        "hidden"  => db.prepare("UPDATE OR IGNORE keywords SET hidden = ? WHERE keyword = ?")?,
        "bare"    => db.prepare("UPDATE OR IGNORE keywords SET true = ? WHERE keyword = ?")?,
        _         => Err("invalid option type")?,
    };

    check_keyword(&db, &keyword, &msg)?;

    if let Ok(1) = statement.execute(&[&value, &keyword]) {
        msg.channel_id.say(&format!("Option `{}` for {} now set to {}", option, keyword, value))?;
    } else {
        msg.channel_id.say(&format!("Could not set `{}` for {}.", option, keyword))?;
    }
});

fn check_keyword(db: &DatabaseConnection, keyword: &str, msg: &Message) -> Result<Option<KeywordEntry>, CommandError> {
    if let Ok(Ok(key)) = db.query_row("SELECT * FROM keywords WHERE keyword=?", &[&keyword], from_row::<KeywordEntry>) {
        if (key.protect || key.hidden) && key.owner != msg.author.id {
            msg.channel_id.say(&format!("Sorry, only {} is cleared for that.", key.owner.mention()))?;
            Err("Unauthorised".into())
        } else {
            Ok(Some(key))
        }
    } else {
        Ok(None)
    }
}
