use crate::schema::*;
use crate::types::*;
use diesel::{prelude::*, pg::upsert::*, result::Error::{NotFound as QueryNotFound}};
use rand::Rng;
use serenity::model::{channel::Message, id::{ChannelId, UserId}};
use serenity::{client::Context, Error, CACHE};

pub fn last_seen_id(ctx: &mut Context, msg: &Message) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get().unwrap();

    let mut seen_ids: Vec<SeenId> = Vec::new();

    let channel_name = match msg.channel_id.to_channel() {
        Ok(Channel::Guild(channel)) => channel.read().name().to_string(),
        Ok(Channel::Group(channel)) => channel.read().name().into_owned(),
        Ok(Channel::Private(channel)) => channel.read().name(),
        Ok(Channel::Category(channel)) => channel.read().name().to_string(),
        Err(_) => String::from("Error"),
    };

    seen_ids.push(SeenId {
        id:   *msg.author.id.as_u64() as i64,
        at:   msg.timestamp,
        kind: "User".to_string(),
        name: msg.author.name.clone(),
    });

    seen_ids.push(SeenId {
        id:   *msg.channel_id.as_u64() as i64,
        at:   msg.timestamp,
        kind: "Channel".to_string(),
        name: channel_name,
    });

    if let Some(guild) = msg.guild_id.and_then(|g| g.to_partial_guild().ok()) {
        seen_ids.push(SeenId {
            id:   *guild.id.as_u64() as i64,
            at:   msg.timestamp,
            kind: "Guild".to_string(),
            name: guild.name,
        });
    }

    if let Some(webhook) = msg.webhook_id.and_then(|g| g.to_webhook().ok()) {
        seen_ids.push(SeenId {
            id:   *webhook.id.as_u64() as i64,
            at:   msg.timestamp,
            kind: "Webhook".to_string(),
            name: webhook.name.unwrap_or_default(),
        });
    }

    if let Err(error) = diesel::insert_into(seen::table)
        .values(&seen_ids)
        .on_conflict(seen::id)
        .do_update()
        .set(seen::id.eq(excluded(seen::id)))
        .execute(&*db)
    {
        log::error!("{}", error);
    }
}

pub fn bareword_handler(ctx: &mut Context, msg: &Message, name: &str) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get().unwrap();

    let result: QueryResult<_> = try {
        let keyword = keywords::table.find(name).filter(keywords::bare.eq(true)).first::<KeywordEntry>(&*db)?;
        let mut entries = definitions::table.filter(definitions::keyword.eq(name)).load::<DefinitionEntry>(&*db)?;

        rand::thread_rng().shuffle(&mut entries);

        if entries.is_empty() {
            Err(QueryNotFound)?;
        }

        CurrentMemory { idx: 0, key: keyword, def: entries }
    };

    if let Ok(ref c) = result {
        let _ = msg.channel_id.send_message(|_| c.definition());
    }
}

pub fn cached_display_name(channel_id: ChannelId, user_id: UserId) -> Result<String, Error> {
    let cache = CACHE.read();

    // If this is a guild channel and the user is a member...
    if let Some(member) = cache.guild_channel(channel_id)
        .and_then(|c| cache.member(c.read().guild_id, user_id)) {
        // ...use their display name...
        return Ok(member.display_name().into_owned());
    }

    // ...otherwise, just use their username.
    Ok(user_id.to_user()?.name)
}

pub fn dynamic_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    let map   = ctx.data.lock();
    let cache = map.get::<PrefixCache>()?;

    msg.channel().and_then(|c| Some(c.id()))
        .and_then(|i| cache.get(&-(i.0 as i64)).filter(|s| !s.is_empty()))
    .or_else(|| msg.guild_id
        .and_then(|i| cache.get(&(i.0 as i64)).filter(|s| !s.is_empty())))
    .cloned()
}

pub trait EpsilonEq<Rhs: Sized = Self>: Sized {
    fn eps_eq(self, other: Rhs) -> bool;
    fn eps_ne(self, other: Rhs) -> bool {
        !self.eps_eq(other)
    }
}

impl EpsilonEq for f64 {
    fn eps_eq(self, other: Self) -> bool {
        self == other || ((self - other).abs() <= std::f64::EPSILON)
    }
}

impl EpsilonEq for f32 {
    fn eps_eq(self, other: Self) -> bool {
        self == other || ((self - other).abs() <= std::f32::EPSILON)
    }
}
