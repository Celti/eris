use crate::model::{DiceCache, MessageExt};
use serenity::framework::standard::{Args, CommandError, Delimiter};
use serenity::model::prelude::*;
use serenity::prelude::*;

pub struct Handler;
impl EventHandler for Handler {
    fn message(&self, ctx: Context, msg: Message) {
        msg.logger(&ctx);
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(s) = ready.shard {
            log::info!("Logged in as '{}' on {}/{}", ready.user.name, s[0], s[1]);
        } else {
            log::info!("Logged in as '{}'", ready.user.name);
        }

        // TODO get name from persistent store
        ctx.set_activity(Activity::playing("with fire."));
    }

    fn reaction_add(&self, ctx: Context, re: Reaction) {
        let bot_id = serenity::utils::with_cache(&ctx.cache, |cache| cache.user.id);

        // Don't respond to our own reactions.
        if bot_id == re.user_id {
            return;
        }

        // Reaction matcher.
        match re.emoji {
            // Reroll dice.
            ReactionType::Unicode(ref x) if x == "🎲" => {
                let mut map = ctx.data.write();
                let cache = map.entry::<DiceCache>().or_insert_with(Default::default);

                if let Err(err) = || -> Result<(), CommandError> {
                    if let Some(expr) = cache.remove(&re.message_id) {
                        re.message(&ctx.http)?.delete_reactions(&ctx)?;

                        let args = Args::new(&expr, &[Delimiter::Single(' ')]);
                        let name = re.user_id.mention();
                        let roll = crate::modules::dice::process_args(args)?;
                        let sent = re.channel_id.send_message(&ctx.http, |m| {
                            m.content(format!("**{} rolled:**{}", name, roll))
                                .reactions(vec!['🎲'])
                        })?;

                        cache.insert(sent.id, expr);
                    } else {
                        log::info!("Die roll is not in message cache.");
                    };
                    Ok(())
                }(){
                    log::error!("error repeating dice roll: {:?}", err);
                }
            }

            // Delete message.
            ReactionType::Unicode(ref x) if x == "❌" => {
                if let Err(err) = || -> Result<(), CommandError> {
                    let msg = re.message(&ctx.http)?;

                    if msg.author.id == bot_id {
                        msg.delete(&ctx)?;
                    };
                    Ok(())
                }(){
                    log::error!("error deleting message: {:?}", err);
                }
            }

            // An unconfigured reaction type.
            r => log::debug!("Unknown ReactionType: {:?}", r),
        }
    }
}
