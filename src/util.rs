use serenity::model::id::{ChannelId, UserId};
use serenity::{Error, CACHE};

pub fn cached_display_name(
    channel_id: ChannelId,
    user_id: UserId,
) -> Result<String, Error> {
    let cache = CACHE.read();

    // If this is a guild channel and the user is a member...
    if let Some(member) = cache
        .guild_channel(channel_id)
        .and_then(|c| cache.member(c.read().guild_id, user_id))
    {
        // ...use their display name...
        return Ok(member.display_name().into_owned());
    }

    // ...otherwise, just use their username.
    Ok(user_id.get()?.name)
}
