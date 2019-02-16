use self::QueryError::{DatabaseError, NotFound};
use crate::db::model::{Attribute, Character};
use crate::db::DB;
use chrono::Utc;
use diesel::result::DatabaseErrorKind::UniqueViolation;
use diesel::result::Error as QueryError;
use failure::{Fail, SyncFailure};
use serenity::model::id::*;
use serenity::Error as SerenityError;

#[derive(Debug, Fail)]
enum TrackError {
    #[fail(display = "Permission denied.")]
    Denied,
    #[fail(display = "Character or attribute exists.")]
    Exists,
    #[fail(display = "Character or attribute not found.")]
    NotFound,
    #[fail(display = "{}", _0)]
    Query(#[cause] QueryError),
    #[fail(display = "{}", _0)]
    Serenity(#[cause] SyncFailure<SerenityError>),
}

cmd!(AddCharacter(_ctx, msg, args)
     aliases: ["track"],
     desc: "Tracks a character's statistics",
     min_args: 1,
{
    let who     = args.single_quoted::<String>()?;
    let comment = args.rest();
    let channel = msg.channel_id.into();
    let owner   = msg.author.id.into();

    let result: Result<(), TrackError> = try {
        match DB.get_character_by_pair(&who, channel) {
            Err(NotFound) => (),
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(_)         => Err(TrackError::Exists)?,
        };

        let content = format!("**[{}]** {} ({})\n`Nothing currently tracked.`", who, comment, msg.timestamp);
        let message = msg.channel_id.say(&content).map_err(|e| TrackError::Serenity(SyncFailure::new(e)))?;
        message.pin().map_err(|e| TrackError::Serenity(SyncFailure::new(e)))?;

        let ch = Character { name: who.clone(), channel, owner, pin: message.id.into() };
        DB.add_character(&ch).map_err(TrackError::Query)?;
    };

    match result {
        Err(TrackError::Denied) | Err(TrackError::NotFound) => unreachable!(),
        Err(TrackError::Exists) => say!(msg.channel_id, "I'm already tracking {}. See the pinned messages.", who),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(()) => say!(msg.channel_id, "Now tracking {}.", who),
    }
});

cmd!(DelCharacter(_ctx, msg, args)
     aliases: ["untrack"],
     desc: "Stops tracking a character",
     min_args: 1,
{
    let who = args.current_quoted().unwrap();

    let result: Result<(), TrackError> = try {
        let ch = match DB.get_character_by_pair(who, msg.channel_id.into()) {
            Err(NotFound) => Err(TrackError::NotFound)?,
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(character) => character,
        };

        if ch.owner != msg.author.id.into():i64 {
            Err(TrackError::Denied)?;
        }

        ChannelId(ch.channel as u64).delete_message(ch.pin as u64).map_err(|e| TrackError::Serenity(SyncFailure::new(e)))?;
        DB.del_character(&ch).map_err(TrackError::Query)?;
    };

    match result {
        Err(TrackError::Denied) => { say!(msg.channel_id, "Sorry, you're not allowed to edit {}.", who); }
        Err(TrackError::Exists) => { unreachable!() }
        Err(TrackError::NotFound) => { say!(msg.channel_id, "Sorry, I'm not tracking {}.", who); }
        Err(TrackError::Query(error)) => { Err(error)?; }
        Err(TrackError::Serenity(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "No longer tracking {}.", who); }
    }
});

cmd!(AddAttribute(_ctx, msg, args)
     aliases: ["add"],
     desc: "Adds a new attribute to a character",
     min_args: 3,
{
    let who     = args.single_quoted::<String>()?;
    let name    = args.single_quoted::<String>()?;
    let value   = args.single::<i32>()?;
    let maximum = args.single::<i32>().unwrap_or(0);
    let comment = args.rest();

    let result: Result<(), TrackError> = try {
        let ch = match DB.get_character_by_pair(&who, msg.channel_id.into()) {
            Err(NotFound) => Err(TrackError::NotFound)?,
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(character) => character,
        };

        if ch.owner != msg.author.id.into():i64 {
            Err(TrackError::Denied)?;
        }

        match DB.add_attribute(&Attribute { name: name.clone(), value, maximum, pin: ch.pin }) {
            Err(DatabaseError(UniqueViolation,_)) => Err(TrackError::Exists)?,
            Err(error) => Err(TrackError::Query(error))?,
            Ok(_) => (),
        };

        update_pin(&ch, &comment)?;
    };

    match result {
        Err(TrackError::Denied) => { say!(msg.channel_id, "Sorry, you're not allowed to edit {}.", who); }
        Err(TrackError::Exists) => { say!(msg.channel_id, "Sorry, I'm already tracking {} for {}.", name, who); }
        Err(TrackError::NotFound) => { say!(msg.channel_id, "Sorry, I'm not tracking {}.", who); }
        Err(TrackError::Query(error)) => { Err(error)?; }
        Err(TrackError::Serenity(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "Now tracking {} for {}.", name, who); }
    }
});

cmd!(DelAttribute(_ctx, msg, args)
     aliases: ["remove"],
     desc: "Removes an attribute from a character",
     min_args: 2,
{
    let who     = args.single_quoted::<String>()?;
    let name    = args.single_quoted::<String>()?;
    let comment = args.rest();

    let result: Result<(), TrackError> = try {
        let ch = match DB.get_character_by_pair(&who, msg.channel_id.into()) {
            Err(NotFound) => Err(TrackError::NotFound)?,
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(character) => character,
        };

        if ch.owner != msg.author.id.into():i64 {
            Err(TrackError::Denied)?;
        }

        match DB.del_attribute(&Attribute { name: name.clone(), value: 0, maximum: 0, pin: ch.pin }) {
            Err(NotFound) => Err(TrackError::Exists)?,
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(_)         => (),
        };

        update_pin(&ch, &comment)?;
    };

    match result {
        Err(TrackError::Denied) => { say!(msg.channel_id, "Sorry, you're not allowed to edit {}.", who); }
        Err(TrackError::Exists) => { say!(msg.channel_id, "Sorry, I'm not tracking {} for {}.", name, who); }
        Err(TrackError::NotFound) => { say!(msg.channel_id, "Sorry, I'm not tracking {}.", who); }
        Err(TrackError::Query(error)) => { Err(error)?; }
        Err(TrackError::Serenity(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "Now tracking {} for {}.", name, who); }
    }
});

cmd!(UpdateAttribute(_ctx, msg, args)
     aliases: ["change"],
     desc: "Updates an attribute for a character",
     min_args: 3,
{
    let who     = args.single_quoted::<String>()?;
    let name    = args.single_quoted::<String>()?;
    let value   = args.single::<i32>()?;
    let maximum = args.single::<i32>();
    let comment = args.rest();

    let result: Result<(), TrackError> = try {
        let ch = match DB.get_character_by_pair(&who, msg.channel_id.into()) {
            Err(NotFound) => Err(TrackError::NotFound)?,
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(character) => character,
        };

        if ch.owner != msg.author.id.into():i64 {
            Err(TrackError::Denied)?;
        }

        let at = match DB.get_attribute_by_pair(&name, ch.pin) {
            Err(NotFound) => Err(TrackError::Exists)?,
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(attribute) => attribute,
        };

        let new = Attribute {
            name: at.name,
            value: value,
            maximum: maximum.unwrap_or(at.maximum),
            pin: at.pin,
        };

        DB.update_attribute(&new).map_err(TrackError::Query)?;
        update_pin(&ch, &comment)?;
    };

    match result {
        Err(TrackError::Denied) => { say!(msg.channel_id, "Sorry, you're not allowed to edit {}.", who); }
        Err(TrackError::Exists) => { say!(msg.channel_id, "Sorry, I'm not tracking {} for {}.", name, who); }
        Err(TrackError::NotFound) => { say!(msg.channel_id, "Sorry, I'm not tracking {}.", who); }
        Err(TrackError::Query(error)) => { Err(error)?; }
        Err(TrackError::Serenity(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "Updated {} for {}.", name, who); }
    }
});

fn update_pin(ch: &Character, comment: &str) -> Result<(), TrackError> {
    let at_v = DB.get_attributes(ch).map_err(TrackError::Query)?;
    let content = {
        if at_v.is_empty() {
            String::from("Nothing currently tracked.")
        } else {
            at_v.iter().fold(String::new(), |s, at| {
                if at.maximum == 0 {
                    format!("{}\n{}: {}", s, at.name, at.value)
                } else {
                    format!("{}\n{}: {}/{}", s, at.name, at.value, at.maximum)
                }
            })
        }
    };

    let content = format!(
        "**[{}]** {} ({})\n`{}`",
        ch.name,
        comment,
        Utc::now(),
        content
    );

    ChannelId(ch.channel as u64)
        .edit_message(ch.pin as u64, |m| m.content(content))
        .map_err(|e| TrackError::Serenity(SyncFailure::new(e)))?;

    Ok(())
}

grp![AddCharacter, DelCharacter, AddAttribute, DelAttribute, UpdateAttribute];
