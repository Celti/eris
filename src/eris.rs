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

                let mut map = if let Some(map) = data.get_mut::<DiceMessages>() {
                    map
                } else {
                    info!("Map is not initialised, returning.");
                    return ();
                };

                let dice = if let Some(dice) = map.get(&re.message_id) {
                    dice.clone()
                } else {
                    info!("Message is not in set, returning.");
                    return ();
                };

                if let Err(e) = roll_and_send(map, re.channel_id, re.user_id, dice) {
                    error!("Failed to repeat die roll: {}", e);
                } else {
                    match re.channel_id.message(re.message_id) {
                        Ok(m) => {
                            if let Err(e) = m.delete_reactions() {
                                warn!("Failed to clear reactions: {}", e);
                            }
                        }
                        Err(e) => warn!("Failed to get original message: {}", e),
                    }
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

    // If this is a guild channel and the user is a member...
    if let Some(channel) = cache.guild_channel(channel_id) {
        if let Some(member) = cache.member(channel.read().unwrap().guild_id, user_id) {
            // ...use their display name...
            return Ok(member.display_name().into_owned());
        }
    }

    // ...otherwise, just use their username.
    Ok(user_id.get()?.name)
}
