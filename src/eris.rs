use serenity::model::*;
use serenity::prelude::*;
use std::collections::HashSet;
use typemap::Key;

/// Keytype for the list of bot owners.
pub struct OwnerList;
impl Key for OwnerList {
    type Value = HashSet<UserId>;
}

/// Keytype for the list of command argument delimiters.
pub struct DelimiterList;
impl Key for DelimiterList {
    type Value = Vec<String>;
}

/// Keytype for the list of command prefixes.
pub struct PrefixList;
impl Key for PrefixList {
    type Value = Vec<String>;
}

/// Required event handler for Serenity clients.
pub struct Handler;
impl EventHandler for Handler {
    /// Logs connection and sets a default presence.
    fn on_ready(&self, context: Context, ready: Ready) {
        info!("Connected as {}", ready.user.name);

        let game   = Some(Game::playing("you all like a fiddle."));
        let status = OnlineStatus::Idle;
        let afk    = false;

        context.set_presence(game, status, afk);
    }
}
