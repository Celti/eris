use serenity::model::prelude::*;
use serenity::prelude::*;
use serenity::utils::{content_safe, ContentSafeOptions};

use crate::ext::EMOJI;
use itertools::Itertools;
use unicode_segmentation::UnicodeSegmentation;

pub trait MessageLogger {
    fn log(&self, ctx: &Context);
}

impl MessageLogger for Message {
    fn log(&self, ctx: &Context) {
        let distinct = self.member(&ctx)
            .map(|m| m.distinct())
            .unwrap_or_else(|| self.author.tag());

        let options = if let Some(id) = self.guild_id {
            ContentSafeOptions::new().display_as_member_from(id)
        } else {
            ContentSafeOptions::new()
        };

        let content = content_safe(&ctx, &self.content, &options)
            .graphemes(true)
            .map(|g| *EMOJI.get(g).unwrap_or(&g))
            .collect::<String>();

        let attachments = if !self.attachments.is_empty() {
            format!("\n[Attachments: {}]", self.attachments.iter().map(|a| &a.url).join(", "))
        } else {
            String::new()
        };

        let embeds = if !self.embeds.is_empty() {
            let embeds = self.embeds.iter().filter_map(|ref e| {
                let video: Option<&str> = e.video.as_ref().map(|v| v.url.as_str());
                let image: Option<&str> = e.image.as_ref().map(|v| v.url.as_str());
                let url:   Option<&str> = e.url.as_ref().map(String::as_str);
                let desc:  Option<&str> = e.description.as_ref().map(String::as_str);
                video.or(image).or(url).or(desc)
            }).join(", ");
            format!("\n[Embeds: {}]", embeds)
        } else {
            String::new()
        };

        use serenity::model::channel::Channel::*;
        match self.channel_id.to_channel(ctx) {
            Ok(Guild(lock)) => {
                let channel = lock.read();
                let lock = channel.guild(&ctx).unwrap();
                let guild = lock.read();
                log::info!(target: "messages", "[{} #{}] <@{}> {}{}{}",
                           guild.name, channel.name(), distinct, content, attachments, embeds);
            }

            Ok(Group(lock)) => {
                let channel = lock.read();
                log::info!(target: "messages", "[{}] <@{}> {}{}{}",
                           channel.name(), distinct, content, attachments, embeds);
            }

            Ok(Private(lock)) => {
                let channel = lock.read();
                log::info!(target: "messages", "[{}] <@{}> {}{}{}",
                           channel.name(), distinct, content, attachments, embeds);
            }

            Ok(_) | Err(_) => {
                log::warn!(target: "messages", "[Unknown Channel: {}] <@{}> {}{}{}",
                           self.channel_id, distinct, content, attachments, embeds);
            }
        }
    }
}
