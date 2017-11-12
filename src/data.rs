use ext::dice::DiceVec;
use serenity::model::MessageId;
use std::collections::HashMap;
use typemap::Key;

pub struct DiceMessages;
impl Key for DiceMessages {
    type Value = HashMap<MessageId, DiceVec>;
}

// TODO persistent store
