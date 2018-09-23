use crate::schema::*;
use crate::types::*;
use diesel::{prelude::*, result::Error::{NotFound as QueryNotFound}};
use rand::distributions::{Distribution, Uniform};
use serenity::{Error, CACHE};

pub fn log_message(msg: &Message) {
    match msg.channel_id.to_channel() {
        Ok(Channel::Guild(channel)) => {
            log::info!(target: "chat",
                "[{} #{}] {} <{}:{}> {}",
                channel.read().guild_id.to_partial_guild().unwrap().name,
                channel.read().name(),
                msg.timestamp,
                msg.author.id,
                cached_display_name(msg.guild_id, msg.author.id).unwrap(),
                msg.content
            );
        }

        Ok(Channel::Group(channel)) => {
            log::info!(target: "chat",
                "[{}] {} <{}:{}> {}",
                channel.read().name(),
                msg.timestamp,
                msg.author.id,
                msg.author.name,
                msg.content
            );
        }

        Ok(Channel::Private(channel)) => {
            log::info!(target: "chat",
                "[{}] {} <{}:{}> {}",
                channel.read().name(),
                msg.timestamp,
                msg.author.id,
                msg.author.name,
                msg.content
            );
        }

        Ok(Channel::Category(channel)) => {
            log::info!(target: "chat",
                "[{}] {} <{}:{}> {}",
                channel.read().name(),
                msg.timestamp,
                msg.author.id,
                msg.author.name,
                msg.content
            );
        }

        Err(_) => {
            log::warn!(target: "chat",
                "[Unknown Channel] {} <{}:{}> {}",
                msg.timestamp,
                msg.author.id,
                msg.author.name,
                msg.content
            );
        }
    }
}

pub fn last_seen_id(ctx: &mut Context, msg: &Message) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get().unwrap();

    let upsert_id = |seen_id: SeenId| {
        diesel::insert_into(seen::table)
            .values(&seen_id)
            .on_conflict(seen::id)
            .do_update()
            .set(&seen_id)
            .execute(&*db)
            .map_err(|e| log::error!("last_seen_id: DB error: {}", e))
            .ok();
    };

    upsert_id(SeenId {
        id:   *msg.author.id.as_u64() as i64,
        at:   msg.timestamp,
        kind: String::from("User"),
        name: msg.author.name.clone(),
    });

    upsert_id(SeenId {
        id:   *msg.channel_id.as_u64() as i64,
        at:   msg.timestamp,
        kind: String::from("Channel"),
        name: msg.channel_id.name().unwrap_or_else(|| String::from("Unknown")),
    });

    if let Some(guild) = msg.guild_id.and_then(|g| g.to_partial_guild().ok()) {
        upsert_id(SeenId {
            id:   -(*guild.id.as_u64() as i64),
            at:   msg.timestamp,
            kind: String::from("Guild"),
            name: guild.name,
        });
    }

    if let Some(webhook) = msg.webhook_id.and_then(|g| g.to_webhook().ok()) {
        upsert_id(SeenId {
            id:   *webhook.id.as_u64() as i64,
            at:   msg.timestamp,
            kind: String::from("Webhook"),
            name: webhook.name.unwrap_or_else(|| String::from("Unknown")),
        });
    }
}

pub fn bareword_handler(ctx: &mut Context, msg: &Message, name: &str) {
    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get().unwrap();

    let result: QueryResult<_> = try {
        let keyword = keywords::table.find(name).filter(keywords::bare.eq(true)).first::<KeywordEntry>(&*db)?;
        let entries = definitions::table.filter(definitions::keyword.eq(name)).load::<DefinitionEntry>(&*db)?;

        if entries.is_empty() {
            Err(QueryNotFound)?;
        }

        let index = Uniform::new(0, entries.len()).sample(&mut rand::thread_rng());

        CurrentMemory { idx: index, key: keyword, def: entries }
    };

    result.map(|c| msg.channel_id.send_message(|_| c.definition()).ok())
        .map_err(|e| log::error!("bareword_handler: DB error: {}", e)).ok();
}

pub fn cached_display_name(guild_id: Option<GuildId>, user_id: UserId) -> Result<String, Error> {
    let cache = CACHE.read();

    // If this is a guild channel and the user is a member, use their display name.
    if let Some(member) = guild_id.and_then(|guild_id| cache.member(guild_id, user_id)) {
        return Ok(member.display_name().into_owned());
    }

    // ...otherwise, just use their username.
    Ok(user_id.to_user()?.name)
}

pub fn dynamic_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    let map = ctx.data.lock();
    let cache = map.get::<PrefixCache>()?;
    let channel_prefix = cache.get(&-(*msg.channel_id.as_u64() as i64)).filter(|s| !s.is_empty());
    let guild_prefix = || msg.guild_id.and_then(|i| cache.get(&(i.0 as i64)).filter(|s| !s.is_empty()));

    channel_prefix.or_else(guild_prefix).cloned()
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
