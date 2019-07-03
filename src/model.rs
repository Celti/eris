use serenity::client::bridge::gateway::ShardManager;
use serenity::model::id::*;
use serenity::model::Permissions;
use serenity::prelude::*;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::Arc;

pub use crate::db::model::*;

pub struct Owner;
impl TypeMapKey for Owner {
    type Value = UserId;
}

pub struct SerenityShardManager;
impl TypeMapKey for SerenityShardManager {
    type Value = Arc<Mutex<ShardManager>>;
}

pub struct DiceCache;
impl TypeMapKey for DiceCache {
    type Value = HashMap<MessageId, String>;
}

pub struct PrefixCache;
impl TypeMapKey for PrefixCache {
    type Value = HashMap<i64, String>;
}

pub trait OptionDeref<T: Deref> {
    fn as_deref(&self) -> Option<&T::Target>;
}

impl<T: Deref> OptionDeref<T> for Option<T> {
    fn as_deref(&self) -> Option<&T::Target> {
        self.as_ref().map(Deref::deref)
    }
}

pub trait PermissionsExt {
    fn pretty(&self) -> String;
}

impl PermissionsExt for Permissions {
    #[allow(clippy::cognitive_complexity)]
    fn pretty(&self) -> String {
        if self.is_empty() {
            return "None".into();
        }

        let mut strings = Vec::new();

        if self.administrator() {
            strings.push("Administrator");
        }

        if self.view_audit_log() {
            strings.push("View Audit Log");
        }

        if self.manage_guild() {
            strings.push("Manage Server");
        }

        if self.manage_roles() {
            strings.push("Manage Roles");
        }

        if self.manage_channels() {
            strings.push("Manage Channels");
        }

        if self.kick_members() {
            strings.push("Kick Members");
        }

        if self.ban_members() {
            strings.push("Ban Members");
        }

        if self.create_invite() {
            strings.push("Create Instant Invite");
        }

        if self.change_nickname() {
            strings.push("Change Nickname");
        }

        if self.manage_nicknames() {
            strings.push("Manage Nicknames");
        }

        if self.manage_emojis() {
            strings.push("Manage Emojis");
        }

        if self.manage_webhooks() {
            strings.push("Manage Webhooks");
        }

        if self.read_messages() {
            strings.push("View Channels");
        }

        if self.send_messages() {
            strings.push("Send Messages");
        }

        if self.send_tts_messages() {
            strings.push("Send TTS Messages");
        }

        if self.manage_messages() {
            strings.push("Manage Messages");
        }

        if self.embed_links() {
            strings.push("Embed Links");
        }

        if self.attach_files() {
            strings.push("Attach Files");
        }

        if self.read_message_history() {
            strings.push("Read Message History");
        }

        if self.mention_everyone() {
            strings.push("Mention Everyone");
        }

        if self.use_external_emojis() {
            strings.push("Use External Emojis");
        }

        if self.add_reactions() {
            strings.push("Add Reactions");
        }

        if self.connect() {
            strings.push("Connect");
        }

        if self.speak() {
            strings.push("Speak");
        }

        if self.mute_members() {
            strings.push("Mute Members");
        }

        if self.deafen_members() {
            strings.push("Deafen");
        }

        if self.move_members() {
            strings.push("Move Members");
        }

        if self.use_vad() {
            strings.push("Use Voice Activity");
        }

        if self.priority_speaker() {
            strings.push("Priority Speaker");
        }

        strings.join(", ")
    }
}
