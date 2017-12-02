use eris::Handler;
use ext::dice::DiceVec;
use serenity::Client;
use serenity::model::{GuildId, MessageId};
use std::collections::HashMap;
use typemap::Key;

pub struct DiceMessages;
impl Key for DiceMessages {
    type Value = HashMap<MessageId, DiceVec>;
}

pub struct GuildPrefixes;
impl Key for GuildPrefixes {
    type Value = HashMap<GuildId, String>;
}

// TODO persistent store
// TODO dump the ShareMap to persistent store on destruction, load on ready
// TODO populate owners, delimiters from store

pub fn init(client: &mut Client<Handler>) {
    let mut data = client.data.lock();

    data.entry::<DiceMessages>().or_insert(HashMap::default());
    data.entry::<GuildPrefixes>().or_insert(HashMap::default());
}
