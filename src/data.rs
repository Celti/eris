use ext::dice::DiceVec;
use serenity::client::Client;
use serenity::model::id::{GuildId, MessageId};
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

pub fn init(client: &mut Client) {
    let mut data = client.data.lock();

    data.entry::<DiceMessages>().or_insert(HashMap::default());
    data.entry::<GuildPrefixes>().or_insert(HashMap::default());
}
