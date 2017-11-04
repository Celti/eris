pub use errors::*;
pub use serenity::model::*;
pub use serenity::prelude::*;

use serenity::CACHE;
use serenity::utils;

use std::collections::HashMap;
use typemap::Key;

use ext::dice::DiceVec;
use commands::random::roll_and_send;

pub struct Handler;
impl EventHandler for Handler {
    fn on_ready(&self, ctx: Context, ready: Ready) {
        ctx.set_game_name("dice with the universe.");
        info!("Connected as {}", ready.user.name);
    }

    fn on_reaction_add(&self, ctx: Context, re: Reaction) {
        if utils::with_cache(|cache| cache.user.id == re.user_id) {
            return ();
        }

        match re.emoji {
            ReactionType::Unicode(ref x) if x == "🎲" => {
                let mut data = ctx.data.lock();

                let mut map = if let Some(m) = data.get_mut::<DiceMessages>() {
                    m
                } else {
                    info!("map is not initialised, returning.");
                    return ();
                };

                let set = if let Some(s) = map.get(&re.message_id) {
                    s.clone()
                } else {
                    info!("Message is not in set, returning.");
                    return ();
                };

                if let Err(e) = roll_and_send(map, re.channel_id, re.user_id, set) {
                    error!("failed to repeat die roll: {}", e);
                }
            }
            r => debug!("Unknown ReactionType: {:?}", r),
        }
    }
}

pub struct DiceMessages;
impl Key for DiceMessages {
    type Value = HashMap<MessageId, DiceVec>;
}

pub fn get_display_name_from_cache(channel_id: ChannelId, user_id: UserId) -> Result<String> {
    let cache = CACHE.read().unwrap();

    let channel = cache.guild_channel(channel_id).ok_or(
        "channel is not in cache",
    )?;

    let member = cache
        .member(channel.read().unwrap().guild_id, user_id)
        .ok_or("member is not in cache")?;

    Ok(member.display_name().into_owned())
}
