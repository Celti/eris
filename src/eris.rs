use serenity::model::*;
use serenity::prelude::*;
use serenity::utils;

pub struct Handler;
impl EventHandler for Handler {
    fn on_ready(&self, ctx: Context, ready: Ready) {
        if let Some(s) = ready.shard {
            info!("Logged in as '{}' on {}/{}", ready.user.name, s[0], s[1]);
        } else {
            info!("Logged in as '{}'", ready.user.name);
        }

        // TODO get name from persistent store
        ctx.set_game_name("to the crowd.");
    }

    fn on_reaction_add(&self, ctx: Context, re: Reaction) {
        use commands::random::roll_and_send;
        use data::DiceMessages;

        // Don't respond to our own reactions.
        if utils::with_cache(|cache| cache.user.id == re.user_id) { return; }

        // Reaction matcher.
        match re.emoji {
            // Reroll dice.
            ReactionType::Unicode(ref x) if x == "🎲" => {
                let mut data = ctx.data.lock();

                let mut map = if let Some(map) = data.get_mut::<DiceMessages>() {
                    map
                } else {
                    warn!("DiceMessages map is not initialised.");
                    return;
                };

                let dice = if let Some(dice) = map.get(&re.message_id) {
                    dice.clone()
                } else {
                    info!("Message is not in DiceMessages map.");
                    return;
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
            // An unconfigured reaction type.
            r => debug!("Unknown ReactionType: {:?}", r),
        }
    }
}
