use failure::Error;
use serenity::CACHE;
use serenity::model::*;
//use serenity::prelude::*;

pub fn cached_display_name(channel_id: ChannelId, user_id: UserId) -> Result<String, Error> {
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

pub fn init_env_logger() {
    use chrono::Local;
    use env_logger::LogBuilder;
    use log::{LogRecord, LogLevelFilter};
    use std::env;
    
    let mut builder = LogBuilder::new();

    let format = |record: &LogRecord| { format!(
        "[{}] {}: {} {}",
        Local::now(),
        record.location().module_path(),
        record.level(),
        record.args()
    )};

    builder.format(format).filter(None, LogLevelFilter::Info);

    if let Ok(var) = env::var("ERIS_LOG") {
       builder.parse(&var);
    }

    builder.init().unwrap();
}

/*
pub fn select_prefix(ctx: &mut Context, msg: &Message) -> Option<String> {
    match msg.guild_id() {
        None => None,
        Some(_) => Some(String::from(".")),
    }
}
*/
