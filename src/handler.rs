use crate::db::BotInfo;
use crate::model::DiceCache;
use crate::logger::MessageLogger;
use serenity::model::prelude::*;
use serenity::prelude::*;
use std::sync::Arc;
use std::collections::HashMap;

pub struct Handler;
impl EventHandler for Handler {
    fn channel_create(&self, _ctx: Context, _channel: Arc<RwLock<GuildChannel>>) {}
    fn category_create(&self, _ctx: Context, _category: Arc<RwLock<ChannelCategory>>) {}
    fn category_delete(&self, _ctx: Context, _category: Arc<RwLock<ChannelCategory>>) {}
    fn private_channel_create(&self, _ctx: Context, _channel: Arc<RwLock<PrivateChannel>>) {}
    fn channel_delete(&self, _ctx: Context, _channel: Arc<RwLock<GuildChannel>>) {}
    fn channel_pins_update(&self, _ctx: Context, _pin: ChannelPinsUpdateEvent) {}
    fn channel_recipient_addition(&self, _ctx: Context, _group_id: ChannelId, _user: User) {}
    fn channel_recipient_removal(&self, _ctx: Context, _group_id: ChannelId, _user: User) {}
    fn channel_update(&self, _ctx: Context, _old: Option<Channel>, _new: Channel) {}

    fn guild_ban_addition(&self, _ctx: Context, _guild_id: GuildId, _banned_user: User) {}
    fn guild_ban_removal(&self, _ctx: Context, _guild_id: GuildId, _unbanned_user: User) {}
    fn guild_create(&self, _ctx: Context, _guild: Guild, _is_new: bool) {}
    fn guild_delete(&self, _ctx: Context, _incomplete: PartialGuild, _full: Option<Arc<RwLock<Guild>>>) {}
    fn guild_emojis_update(&self, _ctx: Context, _guild_id: GuildId, _current_state: HashMap<EmojiId, Emoji>) {}
    fn guild_integrations_update(&self, _ctx: Context, _guild_id: GuildId) {}
    fn guild_member_addition(&self, _ctx: Context, _guild_id: GuildId, _new_member: Member) {}
    fn guild_member_removal(&self, _ctx: Context, _guild: GuildId, _user: User, _member_data_if_available: Option<Member>) {}
    fn guild_member_update(&self, _ctx: Context, _old_if_available: Option<Member>, _new: Member) {}
    fn guild_role_create(&self, _ctx: Context, _guild_id: GuildId, _new: Role) {}
    fn guild_role_delete(&self, _ctx: Context, _guild_id: GuildId, _removed_role_id: RoleId, _removed_role_data_if_available: Option<Role>) {}
    fn guild_role_update(&self, _ctx: Context, _guild_id: GuildId, _old_data_if_available: Option<Role>, _new: Role) {}
    fn guild_unavailable(&self, _ctx: Context, _guild_id: GuildId) {}
    fn guild_update(&self, _ctx: Context, _old_data_if_available: Option<Arc<RwLock<Guild>>>, _new_but_incomplete: PartialGuild) {}

    fn presence_replace(&self, _ctx: Context, _: Vec<Presence>) {}
    fn presence_update(&self, _ctx: Context, _new_data: PresenceUpdateEvent) {}

    fn typing_start(&self, _ctx: Context, _: TypingStartEvent) {}

    fn message_update(&self, _ctx: Context, _old: Option<Message>, _new: Option<Message>, _data: MessageUpdateEvent) {}
    fn message_delete(&self, _ctx: Context, _channel_id: ChannelId, _deleted_message_id: MessageId) {}
    fn message_delete_bulk(&self, _ctx: Context, _channel_id: ChannelId, _multiple_deleted_messages_ids: Vec<MessageId>) {}

    fn reaction_remove(&self, _ctx: Context, _removed_reaction: Reaction) {}
    fn reaction_remove_all(&self, _ctx: Context, _channel_id: ChannelId, _removed_from_message_id: MessageId) {}

    fn message(&self, ctx: Context, msg: Message) {
        msg.log(&ctx);
    }

    fn ready(&self, ctx: Context, ready: Ready) {
        if let Some(s) = ready.shard {
            log::info!("Logged in as '{}' on {}/{}", ready.user.name, s[0], s[1]);
        } else {
            log::info!("Logged in as '{}'", ready.user.name);
        }

        let bot_id = serenity::utils::with_cache(&ctx, |cache| cache.user.id);
        let activity = BotInfo::get_activity(bot_id.into())
            .unwrap_or_else(|_| Activity::playing("with fire"));

        ctx.set_activity(activity);
    }

    fn reaction_add(&self, ctx: Context, re: Reaction) {
        let bot_id = serenity::utils::with_cache(&ctx, |cache| cache.user.id);

        // Don't respond to our own reactions.
        if bot_id == re.user_id {
            return;
        }

        // Reaction matcher.
        match re.emoji {
            // Reroll dice.
            ReactionType::Unicode(ref x) if x == "🎲" => {
                let cached = {
                    let mut data = ctx.data.write();
                    let cache = data.entry::<DiceCache>().or_insert_with(Default::default);
                    cache.remove(&re.message_id)
                };

                if let Some(expr) = cached {
                    err_log!(re.channel_id.delete_reaction(&ctx, re.message_id, None, '🎲'));
                    crate::modules::dice::handle_roll(&ctx, re.channel_id, re.user_id, &expr);
                } else {
                    log::info!("Die roll is not in message cache.");
                };
            }

            // Delete message.
            ReactionType::Unicode(ref x) if x == "❌" => {
                err_log!(re.message(&ctx).and_then(|msg| {
                    if msg.author.id == bot_id {
                        msg.delete(&ctx)
                    } else {
                        Ok(())
                    }
                }));
            }

            // An unconfigured reaction type.
            r => log::debug!("Unknown ReactionType: {:?}", r),
        }
    }
}
