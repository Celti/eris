use serenity::model::{channel::Message, id::{ChannelId, UserId}};
use serenity::{client::Context, Error, CACHE};

pub fn bareword_handler(ctx: &mut Context, msg: &Message, name: &str) {
    use crate::schema::*;
    use crate::types::*;
    use diesel::prelude::*;
    use rand::Rng;
    use diesel::result::Error::{NotFound as QueryNotFound};

    let map    = ctx.data.lock();
    let handle = map.get::<DatabaseHandle>().unwrap();
    let db     = handle.get().unwrap();

    let result: QueryResult<_> = do catch {
        let keyword = keywords::table.find(name).filter(keywords::bare.eq(true)).first::<KeywordEntry>(&*db)?;
        let mut entries = definitions::table.filter(definitions::keyword.eq(name)).load::<DefinitionEntry>(&*db)?;

        rand::thread_rng().shuffle(&mut entries);

        if entries.is_empty() {
            Err(QueryNotFound)?;
        }

        CurrentMemory { idx: 0, key: keyword, def: entries }
    };

    if let Ok(ref c) = result {
        let _ = msg.channel_id.say(c.definition());
    }
}

pub fn cached_display_name(channel_id: ChannelId, user_id: UserId) -> Result<String, Error> {
    let cache = CACHE.read();

    // If this is a guild channel and the user is a member...
    if let Some(member) = cache.guild_channel(channel_id)
        .and_then(|c| cache.member(c.read().guild_id, user_id)) {
        // ...use their display name...
        return Ok(member.display_name().into_owned());
    }

    // ...otherwise, just use their username.
    Ok(user_id.get()?.name)
}

pub trait EpsilonEq<Rhs: Sized = Self>: Sized {
    fn eps_eq(self, other: Rhs) -> bool;
    fn eps_ne(self, other: Rhs) -> bool {
        !self.eps_eq(other)
    }
}

impl EpsilonEq for f64 {
    fn eps_eq(self, other: Self) -> bool {
        self == other || ((self - other).abs() <= std::f64::EPSILON)
    }
}

impl EpsilonEq for f32 {
    fn eps_eq(self, other: Self) -> bool {
        self == other || ((self - other).abs() <= std::f32::EPSILON)
    }
}
