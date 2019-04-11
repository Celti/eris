use chrono::Utc;
use crate::db::CharTrack as DB;
use crate::db::model::{Attribute, Channel, Character, Note};
use diesel::result::Error as QueryError;
use diesel::result::{Error::NotFound, OptionalExtension};
use serenity::model::id::*;
use serenity::model::misc::Mentionable;
use serenity::prelude::*;
use std::error::Error as StdError;
use std::fmt::{Display, Formatter, Result as FmtResult};
use serenity::client::Context;
use serenity::framework::standard::{Args, CommandResult};
use serenity::framework::standard::macros::{command, group};
use serenity::model::channel::Message;

#[derive(Debug)]
enum TrackError {
    Denied,
    Exists,
    Query(QueryError),
    Serenity(Box<SerenityError>),
}

impl Display for TrackError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            TrackError::Denied => write!(f, "Permission denied."),
            TrackError::Exists => write!(f, "Character or attribute already exists."),
            TrackError::Query(q) => write!(f, "{}", q),
            TrackError::Serenity(s) => write!(f, "{}", s),
        }
    }
}

impl From<QueryError> for TrackError {
    fn from(err: QueryError) -> Self {
        TrackError::Query(err)
    }
}

impl From<SerenityError> for TrackError {
    fn from(err: SerenityError) -> Self {
        TrackError::Serenity(Box::new(err))
    }
}

impl StdError for TrackError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            TrackError::Query(q) => Some(q),
            TrackError::Serenity(s) => Some(s),
            _ => None,
        }
    }
}

#[command]
#[description("Track a character's statistics.")]
#[usage(r#""<Name" [Comment}"#)]
#[min_args(1)]
fn track(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let who = args.quoted().single::<String>()?;
    let comment = args.rest();
    let channel = msg.channel_id.into();
    let owner = msg.author.id.into();

    let result = || -> Result<(), TrackError> {
        match DB::get_character_by_pair(&who, channel) {
            Err(NotFound) => (),
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(_)         => Err(TrackError::Exists)?,
        };

        let content = format!("**[{}]** {} ({})\n```New character.```", who, comment, msg.timestamp);
        let message = msg.channel_id.say(&ctx, &content)?;
        message.pin(&ctx)?;

        let ch = Character { name: who.clone(), channel, owner, pin: message.id.into() };
        DB::add_character(&ch)?;

        Ok(())
    }();

    match result {
        Err(TrackError::Denied) => unreachable!(),
        Err(TrackError::Exists) => say!(ctx, msg, "I'm already tracking {}. See the pinned messages.", who),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(_) => say!(ctx, msg, "Now tracking {}.", who),
    }

    Ok(())
}

#[command]
#[description("Stops tracking a character.")]
#[usage(r#""<Name>""#)]
#[num_args(1)]
fn forget(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let who = args.quoted().current().ok_or("No character provided!")?;
    let result = || -> Result<(), TrackError> {
        let ch = DB::get_character_by_pair(who, msg.channel_id.into())?;
        denied(&ch, msg.author.id)?;

        ChannelId(ch.channel as u64).delete_message(&ctx, ch.pin as u64)?;
        DB::del_character(&ch)?;

        Ok(())
    }();

    match result {
        Err(TrackError::Denied) => say!(ctx, msg, "Sorry, you're not allowed to edit {}.", who),
        Err(TrackError::Exists) => unreachable!(),
        Err(TrackError::Query(NotFound)) => say!(ctx, msg, "Sorry, I'm not tracking {}.", who),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(_) => say!(ctx, msg, "No longer tracking {}.", who),
    }

    Ok(())
}

#[command]
#[description("Adds or sets a character attribute.")]
#[usage(r#""<Name>" <Attribute> <Value> [Maximum] [Comment]"#)]
#[min_args(3)]
fn set(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let who     = args.quoted().single::<String>()?;
    let name    = args.quoted().single::<String>()?;
    let value   = args.single::<i32>()?;
    let maximum = args.single::<i32>().ok();
    let comment = args.rest();

    let result = || -> Result<Attribute, TrackError> {
        let ch = DB::get_character_by_pair(&who, msg.channel_id.into())?;

        denied(&ch, msg.author.id)?;

        let at = if let Some(at) = DB::get_attribute(&name, ch.pin).optional()? {
            DB::update_attribute(&Attribute {
                name: name.clone(),
                value,
                maximum: maximum.unwrap_or(at.maximum),
                pin: ch.pin
            })
        } else {
            DB::add_attribute(&Attribute {
                name: name.clone(),
                value,
                maximum: maximum.unwrap_or(0),
                pin: ch.pin
            })
        }?;

        update_pin(&ctx, &ch, &comment)?;

        Ok(at)
    }();

    match result {
        Err(TrackError::Denied) => say!(ctx, msg, "Sorry, you're not allowed to edit {}.", who),
        Err(TrackError::Exists) => unreachable!(),
        Err(TrackError::Query(NotFound)) => say!(ctx, msg, "Sorry, I'm not tracking {}.", who),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(at) => if at.maximum == 0 {
            say!(ctx, msg, "Set {} for {} to {}.", at.name, who, at.value);
        } else {
            say!(ctx, msg, "Set {} for {} to {}/{}.", at.name, who, at.value, at.maximum);
        }
    }

    Ok(())
}

#[command]
#[description("Deletes a character attribute or note.")]
#[usage(r#""<Name>" <Attribute|Note> [Comment]"#)]
#[min_args(2)]
fn del(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let who     = args.quoted().single::<String>()?;
    let name    = args.quoted().single::<String>()?;
    let comment = args.rest();

    let result = || -> Result<(), TrackError> {
        let ch = DB::get_character_by_pair(&who, msg.channel_id.into())?;

        denied(&ch, msg.author.id)?;

        match DB::del_attribute(&Attribute { name: name.clone(), value: 0, maximum: 0, pin: ch.pin }) {
            Err(NotFound) => match DB::del_note(&Note { name: name.clone(), note: String::new(), pin: ch.pin }) {
                Err(NotFound) => Err(TrackError::Exists)?,
                Err(error)    => Err(TrackError::Query(error))?,
                Ok(_)         => (),
            }
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(_)         => (),
        };

        update_pin(&ctx, &ch, &comment)?;

        Ok(())
    }();

    match result {
        Err(TrackError::Denied) => say!(ctx, msg, "Sorry, you're not allowed to edit {}.", who),
        Err(TrackError::Exists) => say!(ctx, msg, "Sorry, I'm not tracking {} for {}.", name, who),
        Err(TrackError::Query(NotFound)) => say!(ctx, msg, "Sorry, I'm not tracking {}.", who),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(_) => say!(ctx, msg, "Stopped tracking {} for {}.", name, who),
    }

    Ok(())
}

#[command]
#[aliases(inc)]
#[description("Adds to a character attribute.")]
#[min_args(3)]
#[usage(r#""<Name>" <Attribute> <Modifier> [Comment]"#)]
fn add(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let who     = args.quoted().single::<String>()?;
    let name    = args.quoted().single::<String>()?;
    let value   = args.single::<i32>()?;
    let comment = args.rest();

    let result = || -> Result<Attribute, TrackError> {
        let ch = DB::get_character_by_pair(&who, msg.channel_id.into())?;
        denied(&ch, msg.author.id)?;

        let mut attr = match DB::get_attribute(&name, ch.pin) {
            Err(NotFound) => Err(TrackError::Exists)?,
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(attribute) => attribute,
        };

        attr.value += value;
        DB::update_attribute(&attr)?;

        update_pin(&ctx, &ch, &comment)?;

        Ok(attr)
    }();

    match result {
        Err(TrackError::Denied) => say!(ctx, msg, "Sorry, you're not allowed to edit {}.", who),
        Err(TrackError::Exists) => say!(ctx, msg, "Sorry, I'm not tracking {} for {}.", name, who),
        Err(TrackError::Query(NotFound)) => say!(ctx, msg, "Sorry, I'm not tracking {}.", who),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(at) => say!(ctx, msg, "Set {} for {} to {}.", at.name, who, at.value),
    }

    Ok(())
}

#[command]
#[aliases(dec)]
#[description("Subtracts from a character attribute.")]
#[min_args(3)]
#[usage(r#""<Name>" <Attribute> <Modifier> [Comment]"#)]
fn sub(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let who     = args.quoted().single::<String>()?;
    let name    = args.quoted().single::<String>()?;
    let value   = args.single::<i32>()?;
    let comment = args.rest();

    let result = || -> Result<Attribute, TrackError> {
        let ch = DB::get_character_by_pair(&who, msg.channel_id.into())?;
        denied(&ch, msg.author.id)?;

        let mut attr= match DB::get_attribute(&name, ch.pin) {
            Err(NotFound) => Err(TrackError::Exists)?,
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(attribute) => attribute,
        };

        attr.value -= value;
        DB::update_attribute(&attr)?;

        update_pin(&ctx, &ch, &comment)?;

        Ok(attr)
    }();

    match result {
        Err(TrackError::Denied) => say!(ctx, msg, "Sorry, you're not allowed to edit {}.", who),
        Err(TrackError::Exists) => say!(ctx, msg, "Sorry, I'm not tracking {} for {}.", name, who),
        Err(TrackError::Query(NotFound)) => say!(ctx, msg, "Sorry, I'm not tracking {}.", who),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(at) => say!(ctx, msg, "Set {} for {} to {}.", at.name, who, at.value),
    }

    Ok(())
}

#[command]
#[description("Adds or edits a character note.")]
#[min_args(3)]
#[usage(r#""<Name>" <Note> <Message...>"#)]
fn note(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let who  = args.quoted().single::<String>()?;
    let name = args.quoted().single::<String>()?;
    let note = args.rest();

    let result = || -> Result<(), TrackError> {
        let ch = DB::get_character_by_pair(&who, msg.channel_id.into())?;
        denied(&ch, msg.author.id)?;

        DB::set_note(&Note { name: name.clone(), note: note.to_string(), pin: ch.pin })?;

        update_pin(&ctx, &ch, "")?;

        Ok(())
    }();

    match result {
        Err(TrackError::Denied) => say!(ctx, msg, "Sorry, you're not allowed to edit {}.", who),
        Err(TrackError::Exists) => unreachable!(),
        Err(TrackError::Query(NotFound)) => say!(ctx, msg, "Sorry, I'm not tracking {}.", who),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(_) => say!(ctx, msg, "Added note on {} for {}.", name, who),
    }

    Ok(())
}

#[command]
#[aliases(gm)]
#[description("(Un)sets the current user as the channel GM.")]
fn claim(ctx: &mut Context, msg: &Message) -> CommandResult {
    let channel: i64 = msg.channel_id.into();
    let gm: i64 = msg.author.id.into();

    let result = || -> Result<(), TrackError> {
        match DB::get_channel(channel) {
            Err(NotFound) => { DB::add_channel(&Channel{channel, gm})?; }
            Err(error) => Err(TrackError::Query(error))?,
            Ok(ch) => {
                if ch.gm == gm {
                    DB::del_channel(&Channel{channel, gm})?;
                } else {
                    Err(TrackError::Denied)?;
                }
            }
        };

        Ok(())
    }();

    match result {
        Err(TrackError::Denied) => say!(ctx, msg, "Sorry, {} already has a GM.", msg.channel_id.mention()),
        Err(TrackError::Exists) => unreachable!(),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(_) => say!(ctx, msg, "Updated GM for {}.", msg.channel_id.mention()),
    }

    Ok(())
}

#[command]
#[description("Recreates the character tracking message.")]
#[min_args(1)]
#[usage(r#""<Name>" [Comment]"#)]
fn reload(ctx: &mut Context, msg: &Message, mut args: Args) -> CommandResult {
    let who     = args.quoted().single::<String>()?;
    let comment = args.rest();

    let result = || -> Result<(), TrackError> {
        let old = DB::get_character_by_pair(&who, msg.channel_id.into())?;
        denied(&old, msg.author.id)?;

        let content = format!("**[{}]** {} ({})\n```Regenerating character...```", who, comment, msg.timestamp);
        let message = msg.channel_id.say(&ctx, &content)?;
        message.pin(&ctx)?;
        ChannelId(old.channel as u64).delete_message(&ctx, old.pin as u64).ok();

        let new = Character { name: old.name.clone(), channel: old.channel, owner: old.owner, pin: message.id.into() };
        DB::update_pin(&old, &new)?;
        update_pin(&ctx, &new, comment)?;

        Ok(())
    }();

    match result {
        Err(TrackError::Denied) => say!(ctx, msg, "Sorry, you're not allowed to edit {}.", who),
        Err(TrackError::Exists) => say!(ctx, msg, "I'm already tracking {}. See the pinned messages.", who),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(_) => say!(ctx, msg, "Now tracking {}.", who),
    }

    Ok(())
}

fn denied(ch: &Character, id: UserId) -> Result<(), TrackError> {
    let user: i64 = id.into();

    match DB::get_channel(ch.channel) {
        Ok(channel) => if user != channel.gm && user != ch.owner {
            Err(TrackError::Denied)?;
        },
        Err(NotFound) => if user != ch.owner {
            Err(TrackError::Denied)?;
        },
        Err(error) => Err(TrackError::Query(error))?,
    };

    Ok(())
}

fn update_pin(ctx: &Context, ch: &Character, comment: &str) -> Result<(), TrackError> {
    let attrs = DB::get_attributes(ch)?;
    let notes = DB::get_notes(ch)?;

    let attrs = {
        if attrs.is_empty() {
            String::from("Nothing currently tracked.")
        } else {
            attrs.iter().fold(String::new(), |s, at| {
                if at.maximum == 0 {
                    format!("{}\n{}: {}", s, at.name, at.value)
                } else {
                    format!("{}\n{}: {}/{}", s, at.name, at.value, at.maximum)
                }
            })
        }
    };

    let notes = {
        if notes.is_empty() {
            String::from("No notes.")
        } else {
            notes.iter().fold(String::new(), |s, n| format!("{}\n{}: {}", s, n.name, n.note))
        }
    };

    let content = format!("**[{}]** {} ({})\n```{}\n{}```", ch.name, comment, Utc::now(), attrs, notes);

    ChannelId(ch.channel as u64).edit_message(&ctx, ch.pin as u64, |m| m.content(content))?;

    Ok(())
}

group!({
    name: "tracker",
    options: { prefix: "ct" },
    commands: [track, forget, set, note, del, add, sub, claim, reload]
});
