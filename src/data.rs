use ext::dice::DiceVec;
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
