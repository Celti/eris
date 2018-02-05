use serenity::client::Context;
use serenity::model::channel::Message;
use serenity::model::id::{ChannelId, UserId};
use serenity::{CACHE, Result};

pub fn cached_display_name(channel_id: ChannelId, user_id: UserId) -> Result<String> {
    let cache = CACHE.read();

    // If this is a guild channel and the user is a member...
    if let Some(channel) = cache.guild_channel(channel_id) {
        if let Some(member) = cache.member(channel.read().guild_id, user_id) {
            // ...use their display name...
            return Ok(member.display_name().into_owned());
        }
    }

    // ...otherwise, just use their username.
    Ok(user_id.get()?.name)
}

pub fn select_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    use data::GuildPrefixes;
    use std::collections::HashMap;

    let mut data = ctx.data.lock();
    let map = data.entry::<GuildPrefixes>().or_insert(HashMap::default());

    map.get(&msg.guild_id()?).cloned()
}
