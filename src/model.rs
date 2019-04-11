use itertools::Itertools;
use serenity::client::bridge::gateway::ShardManager;
use serenity::model::channel::Message;
use serenity::model::id::*;
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
    fn distinct(&self, ctx: &Context) -> String;
    fn to_logstr(&self, ctx: &Context) -> String;
    fn logger(&self, ctx: &Context);
    fn link(&self) -> String;
}

impl MessageExt for Message {
    fn distinct(&self, ctx: &Context) -> String {
        self.guild_id
            .and_then(|guild_id| {
                ctx.cache
                    .read()
                    .member(guild_id, self.author.id)
                    .map(|member| member.distinct())
            })
            .unwrap_or_else(|| self.author.tag())
    }

    fn logger(&self, ctx: &Context) {
        use serenity::model::channel::Channel::*;
        match self.channel_id.to_channel(&ctx) {
            Ok(Guild(lock)) => {
                let channel = lock.read();
                let lock = channel.guild(&ctx).unwrap();
                let guild = lock.read();
                log::info!(target: "messages", "[{} #{}] {}", guild.name, channel.name(), self.to_logstr(&ctx));
            }

            Ok(Group(lock)) => {
                let channel = lock.read();
                log::info!(target: "messages", "[{}] {}", channel.name(), self.to_logstr(&ctx));
            }

            Ok(Private(lock)) => {
                let channel = lock.read();
                log::info!(target: "messages", "[{}] {}", channel.name(), self.to_logstr(&ctx));
            }

            Ok(Category(_)) | Err(_) => {
                log::warn!(target: "messages", "[Unknown Channel: {}] {}",
                           self.channel_id,
                           self.to_logstr(&ctx));
            }
        }
    }

    fn to_logstr(&self, ctx: &Context) -> String {
        let content = serenity::utils::content_safe(&ctx, &self.content, &Default::default());
        let content = content.graphemes(true).map(|g| *EMOJI.get(g).unwrap_or(&g)).collect::<String>();

        let attachments = self.attachments.iter().map(|a| &a.url).join(", ");
        let attachments = if !attachments.is_empty() {
            format!("\n[Attachments: {}]", attachments)
        } else {
            String::new()
        };

        let embeds = self.embeds.iter().filter_map(|ref e| {
            let video: Option<&str> = e.video.as_ref().map(|v| v.url.as_str());
            let image: Option<&str> = e.image.as_ref().map(|v| v.url.as_str());
            let url: Option<&str> = e.url.as_ref().map(String::as_str);
            let desc: Option<&str> = e.description.as_ref().map(String::as_str);
            video.or(image).or(url).or(desc)
        }).join(", ");

        let embeds = if !embeds.is_empty() {
            format!("\n[Embeds: {}]", embeds)
        } else {
            String::new()
        };

        format!("<@{}> {}{}{}", self.distinct(&ctx), content, attachments, embeds)
    }

    fn link(&self) -> String {
        format!(
            "https://discordapp.com/channels/{}/{}/{}",
            self.guild_id
                .map(|id| id.to_string())
                .unwrap_or_else(|| String::from("@me")),
            self.channel_id,
            self.id
        )
    }
}
