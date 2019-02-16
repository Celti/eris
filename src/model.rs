use itertools::Itertools;
use serenity::CACHE;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::id::*;
use serenity::model::channel::Message;
use serenity::prelude::*;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;
use unicode_segmentation::UnicodeSegmentation;

use crate::ext::EMOJI;

pub use crate::db::model::*;

pub struct Owner;
impl TypeMapKey for Owner {
    type Value = UserId;
}

pub struct SerenityShardManager;
impl TypeMapKey for SerenityShardManager {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct DiceCache;
impl TypeMapKey for DiceCache {
    type Value = HashMap<MessageId, String>;
}

pub struct PrefixCache;
impl TypeMapKey for PrefixCache {
    type Value = HashMap<i64, String>;
}

pub trait OptionDeref<T: Deref> {
    fn as_deref(&self) -> Option<&T::Target>;
}

impl<T: Deref> OptionDeref<T> for Option<T> {
    fn as_deref(&self) -> Option<&T::Target> {
        self.as_ref().map(Deref::deref)
    }
}

pub trait MessageExt {
    fn distinct(&self) -> String;
    fn content_safe_all(&self) -> String;
    fn to_logstr(&self) -> String;
    fn logger(&self);
}

impl MessageExt for Message {
    fn distinct(&self) -> String {
        self.guild_id
            .and_then(|guild_id| {
                CACHE
                    .read()
                    .member(guild_id, self.author.id)
                    .map(|member| member.distinct())
            }).unwrap_or_else(|| self.author.tag())
    }

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

        let content = RE.replace_all(&self.content, replacer).into_owned();
        let mut content = content.graphemes(true).map(|g| *EMOJI.get(g).unwrap_or(&g)).collect::<String>();

        let attachments = self.attachments.iter().map(|a| &a.url).join(", ");
        let embeds = self.embeds.iter().filter_map(|ref e| {
            let video: Option<&str> = e.video.as_ref().map(|v| v.url.as_str());
            let image: Option<&str> = e.image.as_ref().map(|v| v.url.as_str());
            let url  : Option<&str> = e.url.as_ref().map(String::as_str);
            let desc : Option<&str> = e.description.as_ref().map(String::as_str);
            video.or(image).or(url).or(desc)
        }).join(", ");

        if ! attachments.is_empty() {
            content.push_str(&format!("\n[Attachments: {}]", attachments));
        }

        if ! embeds.is_empty() {
            content.push_str(&format!("\n[Embeds: {}]", embeds));
        }

        content
    }

    fn logger(&self) {
        use serenity::model::channel::Channel::*;
        match self.channel_id.to_channel_cached() {
            Some(Guild(lock)) => {
                let channel = lock.read();
                let lock    = channel.guild().unwrap();
                let guild   = lock.read();
                log::info!(target: "messages", "[{} #{}] {}", guild.name, channel.name(), self.to_logstr());
            }

            Some(Group(lock)) => {
                let channel = lock.read();
                log::info!(target: "messages", "[{}] {}", channel.name(), self.to_logstr());
            }

            Some(Private(lock)) => {
                let channel = lock.read();
                log::info!(target: "messages", "[{}] {}", channel.name(), self.to_logstr());
            }

            Some(Category(_)) | None => {
                log::warn!(target: "messages", "[Unknown Channel: {}] {}",
                           self.channel_id,
                           self.to_logstr());
            }
        }
    }

    fn to_logstr(&self) -> String {
        format!("<@{}> {}", self.distinct(), self.content_safe_all())
    }
}
