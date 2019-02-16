use crate::db::model::{Attribute, Character, Channel};
use crate::db::DB;
use chrono::Utc;
use diesel::result::Error as QueryError;
use diesel::result::{Error::NotFound, OptionalExtension};
use failure::{Fail, SyncFailure};
use serenity::model::id::*;
use serenity::model::misc::Mentionable;
use serenity::Error as SerenityError;

#[derive(Debug, Fail)]
enum TrackError {
    #[fail(display = "Permission denied.")]
    Denied,
    #[fail(display = "Character or attribute exists.")]
    Exists,
    #[fail(display = "{}", _0)]
    Query(#[cause] QueryError),
    #[fail(display = "{}", _0)]
    Serenity(#[cause] SyncFailure<SerenityError>),
}

cmd!(TrackCharacter(_ctx, msg, args)
     aliases: ["track"],
     desc: "Begins tracking a character's statistics.",
     usage: r#""<Name>" [Comment]"#,
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
        Err(TrackError::Denied) => unreachable!(),
        Err(TrackError::Exists) => say!(msg.channel_id, "I'm already tracking {}. See the pinned messages.", who),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(()) => say!(msg.channel_id, "Now tracking {}.", who),
    }
});

cmd!(ForgetCharacter(_ctx, msg, args)
     aliases: ["forget"],
     desc: "Stops tracking a character.",
     usage: r#""<Character>""#,
     num_args: 1,
{
    let who = args.current_quoted().unwrap();

    let result: Result<(), TrackError> = try {
        let ch = DB.get_character_by_pair(&who, msg.channel_id.into()).map_err(TrackError::Query)?;
        denied(&ch, msg.author.id)?;

        ChannelId(ch.channel as u64).delete_message(ch.pin as u64).map_err(|e| TrackError::Serenity(SyncFailure::new(e)))?;
        DB.del_character(&ch).map_err(TrackError::Query)?;
    };

    match result {
        Err(TrackError::Denied) => { say!(msg.channel_id, "Sorry, you're not allowed to edit {}.", who); }
        Err(TrackError::Exists) => { unreachable!() }
        Err(TrackError::Query(NotFound)) => { say!(msg.channel_id, "Sorry, I'm not tracking {}.", who); }
        Err(TrackError::Query(error)) => { Err(error)?; }
        Err(TrackError::Serenity(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "No longer tracking {}.", who); }
    }
});

cmd!(SetAttribute(_ctx, msg, args)
     aliases: ["set"],
     desc: "Adds or changes a character's attribute.",
     usage: r#""<Character>" <Attribute> <Value> [Maximum] [Comment]"#,
     min_args: 3,
{
    let who     = args.single_quoted::<String>()?;
    let name    = args.single_quoted::<String>()?;
    let value   = args.single::<i32>()?;
    let maximum = args.single::<i32>().ok();
    let comment = args.rest();

    let result: Result<(), TrackError> = try {
        let ch = DB.get_character_by_pair(&who, msg.channel_id.into()).map_err(TrackError::Query)?;

        denied(&ch, msg.author.id)?;

        if let Some(at) = DB.get_attribute_by_pair(&name, ch.pin).optional().map_err(TrackError::Query)? {
            DB.update_attribute(&Attribute {
                name: name.clone(),
                value: value,
                maximum: maximum.unwrap_or(at.maximum),
                pin: ch.pin
            })
        } else {
            DB.add_attribute(&Attribute {
                name: name.clone(),
                value: value,
                maximum: maximum.unwrap_or(0),
                pin: ch.pin
            })
        }.map_err(TrackError::Query)?;

        update_pin(&ch, &comment)?;
    };

    match result {
        Err(TrackError::Denied) => { say!(msg.channel_id, "Sorry, you're not allowed to edit {}.", who); }
        Err(TrackError::Exists) => { unreachable!(); }
        Err(TrackError::Query(NotFound)) => { say!(msg.channel_id, "Sorry, I'm not tracking {}.", who); }
        Err(TrackError::Query(error)) => { Err(error)?; }
        Err(TrackError::Serenity(error)) => { Err(error)?; }
        Ok(()) => match maximum {
            None | Some(0) => say!(msg.channel_id, "Set {} for {} to {}.", name, who, value),
            Some(max) => say!(msg.channel_id, "Set {} for {} to {}/{}.", name, who, value, max),
        }
    }
});

cmd!(DelAttribute(_ctx, msg, args)
     aliases: ["del"],
     desc: "Deletes an attribute from a character.",
     usage: r#""<Character>" <Attribute> [Comment]"#,
     min_args: 2,
{
    let who     = args.single_quoted::<String>()?;
    let name    = args.single_quoted::<String>()?;
    let comment = args.rest();

    let result: Result<(), TrackError> = try {
        let ch = DB.get_character_by_pair(&who, msg.channel_id.into()).map_err(TrackError::Query)?;

        denied(&ch, msg.author.id)?;

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
        Err(TrackError::Query(NotFound)) => { say!(msg.channel_id, "Sorry, I'm not tracking {}.", who); }
        Err(TrackError::Query(error)) => { Err(error)?; }
        Err(TrackError::Serenity(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "Now tracking {} for {}.", name, who); }
    }
});

cmd!(IncAttribute(_ctx, msg, args)
     aliases: ["add", "inc"],
     desc: "Increases the current value for a character's attribute.",
     usage: r#""<Character>" <Attribute> <Modifier> [Comment]"#,
     min_args: 3,
{
    let who     = args.single_quoted::<String>()?;
    let name    = args.single_quoted::<String>()?;
    let value   = args.single::<i32>()?;
    let comment = args.rest();

    let result: Result<(), TrackError> = try {
        let ch = DB.get_character_by_pair(&who, msg.channel_id.into()).map_err(TrackError::Query)?;
        denied(&ch, msg.author.id)?;

        let mut attr= match DB.get_attribute_by_pair(&name, ch.pin) {
            Err(NotFound) => Err(TrackError::Exists)?,
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(attribute) => attribute,
        };

        attr.value += value;

        DB.update_attribute(&attr).map_err(TrackError::Query)?;
        update_pin(&ch, &comment)?;
    };

    match result {
        Err(TrackError::Denied) => { say!(msg.channel_id, "Sorry, you're not allowed to edit {}.", who); }
        Err(TrackError::Exists) => { say!(msg.channel_id, "Sorry, I'm not tracking {} for {}.", name, who); }
        Err(TrackError::Query(NotFound)) => { say!(msg.channel_id, "Sorry, I'm not tracking {}.", who); }
        Err(TrackError::Query(error)) => { Err(error)?; }
        Err(TrackError::Serenity(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "Increased {} by {} for {}.", name, value, who); }
    }
});

cmd!(DecAttribute(_ctx, msg, args)
     aliases: ["sub", "dec"],
     desc: "Decreases the current value for a character's attribute.",
     usage: r#""<Character>" <Attribute> <Modifier> [Comment]"#,
     min_args: 3,
{
    let who     = args.single_quoted::<String>()?;
    let name    = args.single_quoted::<String>()?;
    let value   = args.single::<i32>()?;
    let comment = args.rest();

    let result: Result<(), TrackError> = try {
        let ch = DB.get_character_by_pair(&who, msg.channel_id.into()).map_err(TrackError::Query)?;
        denied(&ch, msg.author.id)?;

        let mut attr= match DB.get_attribute_by_pair(&name, ch.pin) {
            Err(NotFound) => Err(TrackError::Exists)?,
            Err(error)    => Err(TrackError::Query(error))?,
            Ok(attribute) => attribute,
        };

        attr.value -= value;

        DB.update_attribute(&attr).map_err(TrackError::Query)?;
        update_pin(&ch, &comment)?;
    };

    match result {
        Err(TrackError::Denied) => { say!(msg.channel_id, "Sorry, you're not allowed to edit {}.", who); }
        Err(TrackError::Exists) => { say!(msg.channel_id, "Sorry, I'm not tracking {} for {}.", name, who); }
        Err(TrackError::Query(NotFound)) => { say!(msg.channel_id, "Sorry, I'm not tracking {}.", who); }
        Err(TrackError::Query(error)) => { Err(error)?; }
        Err(TrackError::Serenity(error)) => { Err(error)?; }
        Ok(()) => { say!(msg.channel_id, "Decreased {} by {} for {}.", name, value, who); }
    }
});

cmd!(ChannelGM(_ctx, msg)
     aliases: ["gm", "claim"],
     desc: "Sets or removes the current user as the channel GM.",
     max_args: 1,
{
    let channel = msg.channel_id.into():i64;
    let gm = msg.author.id.into():i64;

    let result: Result<(), TrackError> = try {
        match DB.get_channel(channel) {
            Err(NotFound) => {
                DB.set_channel(&Channel{channel, gm}).map_err(TrackError::Query)?;
            }
            Ok(ch) => {
                if ch.gm == gm {
                    DB.del_channel(&Channel{channel, gm}).map_err(TrackError::Query)?;
                } else {
                    Err(TrackError::Denied)?;
                }
            }
            Err(error) => Err(TrackError::Query(error))?,
        };
    };

    match result {
        Err(TrackError::Denied) => say!(msg.channel_id, "Sorry, {} already has a GM.", msg.channel_id.mention()),
        Err(TrackError::Exists) => unreachable!(),
        Err(TrackError::Query(error)) => Err(error)?,
        Err(TrackError::Serenity(error)) => Err(error)?,
        Ok(()) => say!(msg.channel_id, "Updated GM for {}.", msg.channel_id.mention()),
    }
});

fn denied(ch: &Character, id: UserId) -> Result<(), TrackError> {
    let user: i64 = id.into();

    match DB.get_channel(ch.channel) {
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

grp![TrackCharacter, ForgetCharacter, SetAttribute, DelAttribute, IncAttribute, DecAttribute, ChannelGM];
