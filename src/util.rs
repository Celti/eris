use crate::schema::*;
use crate::types::*;
use diesel::{prelude::*, result::Error::{NotFound as QueryNotFound}};
use itertools::Itertools;
use rand::distributions::{Distribution, Uniform};
use serenity::CACHE;

pub fn log_message(msg: &Message) {
    match msg.channel_id.to_channel() {
        Ok(Channel::Guild(channel)) => {
            log::info!(target: "chat",
                "[{} #{}] {}",
                channel.read().guild_id.to_partial_guild().unwrap().name,
                channel.read().name(),
                msg.to_logstr()
            );
        }

        Ok(Channel::Group(channel)) => {
            log::info!(target: "chat",
                "[{}] {}",
                channel.read().name(),
                msg.to_logstr()
            );
        }

        Ok(Channel::Private(channel)) => {
            log::info!(target: "chat",
                "[{}] {}",
                channel.read().name(),
                msg.to_logstr()
            );
        }

        Ok(Channel::Category(_)) => {
            log::warn!(target: "chat",
                "[Channel Category: {}] {}",
                msg.channel_id,
                msg.to_logstr()
            );
        }

        Err(_) => {
            log::warn!(target: "chat",
                "[Unknown Channel: {}] {}",
                msg.channel_id,
                msg.to_logstr()
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

pub fn dynamic_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    let map = ctx.data.lock();
    let cache = map.get::<PrefixCache>()?;
    let channel_prefix = cache.get(&-(*msg.channel_id.as_u64() as i64)).filter(|s| !s.is_empty());
    let guild_prefix = || msg.guild_id.and_then(|i| cache.get(&(*i.as_u64() as i64)).filter(|s| !s.is_empty()));

    channel_prefix.or_else(guild_prefix).cloned()
}

pub trait UserExt {
    fn distinct(&self, id: Option<GuildId>) -> String;
}

impl UserExt for User {
    fn distinct(&self, id: Option<GuildId>) -> String {
        id.and_then(|id| CACHE.read().member(id, &self.id).map(|m| m.distinct()))
            .unwrap_or_else(|| self.tag())
    }
}

pub trait MessageExt {
    fn content_safe_all(&self) -> String;
    fn to_logstr(&self) -> String;
}

impl MessageExt for Message {
    fn content_safe_all(&self) -> String {
        use serenity::utils::{parse_channel, parse_emoji, parse_role, parse_username};
        use lazy_static::lazy_static;
        use regex::{Captures, Regex};

        lazy_static! {
            static ref RE: Regex = Regex::new(r"(?x)
                (?P<user>    <@ !? [[:digit:]]+ >               ) |
                (?P<role>    <@ &  [[:digit:]]+ >               ) |
                (?P<channel> <\#   [[:digit:]]+ >               ) |
                (?P<emoji>   <a? : [[:word:]]+ : [[:digit:]]+ > )")
                .unwrap();
        }

        let replacer = |cap: &Captures| -> String {
            if let Some(mention) = cap.name("user") {
                parse_username(mention.as_str())
                    .map(UserId)
                    .and_then(|id| id.to_user_cached())
                    .map_or(String::from("@deleted-user#0000"), |u| {
                        format!("@{}", u.read().tag())
                    })
            } else if let Some(mention) = cap.name("role") {
                parse_role(mention.as_str())
                    .map(RoleId)
                    .and_then(|id| id.to_role_cached())
                    .map_or(String::from("@deleted-role"), |r| format!("@{}", r.name))
            } else if let Some(mention) = cap.name("channel") {
                parse_channel(mention.as_str())
                    .map(ChannelId)
                    .and_then(|id| id.name())
                    .map_or(String::from("#deleted-channel"), |n| format!("#{}", n))
            } else if let Some(mention) = cap.name("emoji") {
                parse_emoji(mention.as_str()).map_or(String::from(":deleted-emoji:"), |e| {
                    format!("![{}]({})", e.name, e.url())
                })
            } else {
                String::new()
            }
        };

        let content     = RE.replace_all(&self.content, replacer).into_owned();
        let attach_urls = self.attachments.iter().map(|a| &a.url).join("\n");
        let embed_urls  = self.embeds.iter().filter_map(|e| e.url.as_ref()).join("\n");
        let video_urls  = self.embeds.iter().filter_map(|e| e.video.as_ref()).map(|v| &v.url).join("\n");
        let image_urls  = self.embeds.iter().filter_map(|e| e.image.as_ref()).map(|i| &i.url).join("\n");

        [content, attach_urls, image_urls, video_urls, embed_urls].iter().filter(|s| !s.is_empty()).join("\n")
    }

    fn to_logstr(&self) -> String {
        format!("{} <@{}> {}",
                self.timestamp.format("%H:%M:%S"),
                self.author.distinct(self.guild_id),
                self.content_safe_all())
    }
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
