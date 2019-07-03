#[allow(clippy::identity_conversion)]
mod schema;
pub mod model;

use crate::db::model::*;
use crate::db::schema::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use diesel::result::Error as QueryError;
use lazy_static::lazy_static;
use serenity::model::gateway::Activity;
use std::collections::HashMap;

lazy_static! {
    static ref DB: Database = Database::connect();
}

no_arg_sql_function!(random, (), "Represents the SQL RANDOM() function");

struct Database {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Database {
    fn connect() -> Self {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let manager = ConnectionManager::<PgConnection>::new(url);
        let pool = Pool::new(manager).expect("database connection pool");

        Self { pool }
    }

    fn get(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.clone().get().expect("database connection")
    }
}

pub struct DynamicPrefix;
impl DynamicPrefix {
    pub fn get() -> QueryResult<HashMap<i64, String>> {
        Ok(prefixes::table.load(&DB.get())?.into_iter().collect())
    }

    pub fn set(prefix: &Prefix) -> QueryResult<Prefix> {
        Ok(diesel::insert_into(prefixes::table)
            .values(prefix)
            .on_conflict(prefixes::id)
            .do_update()
            .set(prefix)
            .get_result(&DB.get())?)
    }
}

pub struct Memory;
impl Memory {
    pub fn get_keyword(kw: &str) -> QueryResult<Keyword> {
        Ok(keywords::table.find(kw).first(&DB.get())?)
    }

    pub fn add_keyword(kw: &Keyword) -> QueryResult<Keyword> {
        Ok(diesel::insert_into(keywords::table)
            .values(kw)
            .get_result(&DB.get())?)
    }

    pub fn update_keyword(kw: &Keyword) -> QueryResult<Keyword> {
        Ok(diesel::update(kw).set(kw).get_result(&DB.get())?)
        // Ok(kw.save_changes(&DB.get())?)
    }

    // pub fn del_keyword(kw: &Keyword) -> QueryResult<Keyword> {
    //     Ok(diesel::delete(kw).get_result(&DB.get())?)
    // }

    pub fn find_keywords(partial: &str) -> QueryResult<Vec<Keyword>> {
        let find = format!("%{}%", partial);
        Ok(keywords::table
            .filter(keywords::keyword.ilike(&find))
            .get_results(&DB.get())?)
    }

    pub fn get_bareword(word: &str) -> QueryResult<Definition> {
        let kw = keywords::table
            .find(word)
            .filter(keywords::bareword.eq(true))
            .first::<Keyword>(&DB.get())?;

        Ok(Definition::belonging_to(&kw)
            .order_by(random)
            .first(&DB.get())?)
    }

    pub fn get_definition(kw: &str, def: &str) -> QueryResult<Definition> {
        Ok(definitions::table.find((kw, def)).first(&DB.get())?)
    }

    pub fn get_definitions(kw: &Keyword) -> QueryResult<Vec<Definition>> {
        Ok(Definition::belonging_to(kw).get_results(&DB.get())?)
    }

    pub fn add_definition(def: &Definition) -> QueryResult<Definition> {
        Ok(diesel::insert_into(definitions::table)
            .values(def)
            .get_result(&DB.get())?)
    }

    pub fn del_definition(def: &Definition) -> QueryResult<Definition> {
        Ok(diesel::delete(def).get_result(&DB.get())?)
    }

    pub fn find_definitions(kw: &Keyword, partial: &str) -> QueryResult<Vec<Definition>> {
        let find = format!("%{}%", partial);
        Ok(Definition::belonging_to(kw)
            .filter(definitions::definition.ilike(&find))
            .get_results(&DB.get())?)
    }
}

pub struct CharTrack;
impl CharTrack {
    pub fn update_pin(old: &Character, new: &Character) -> QueryResult<Character> {
        let db = DB.get();
        let temp = Character {
            name: new.name.clone(),
            owner: new.owner,
            pin: new.pin,
            channel: 0
        };

        db.transaction::<Character,QueryError,_>(|| {
            diesel::insert_into(characters::table).values(&temp).execute(&db)?;
            diesel::update(Attribute::belonging_to(old)).set(attributes::pin.eq(new.pin)).execute(&db)?;
            diesel::update(Note::belonging_to(old)).set(notes::pin.eq(new.pin)).execute(&db)?;
            diesel::delete(old).execute(&db)?;
            diesel::update(&temp).set(new).get_result(&db)
        })
    }

    pub fn add_character(ch: &Character) -> QueryResult<Character> {
        Ok(diesel::insert_into(characters::table)
            .values(ch)
            .get_result(&DB.get())?)
    }

    pub fn del_character(ch: &Character) -> QueryResult<Character> {
        Ok(diesel::delete(ch).get_result(&DB.get())?)
    }

    // pub fn get_character(pin: i64) -> QueryResult<Character> {
    //     Ok(characters::table.find(pin).first(&DB.get())?)
    // }

    pub fn get_character_by_pair(name: &str, channel: i64) -> QueryResult<Character> {
        Ok(characters::table
            .filter(characters::name.eq(name))
            .filter(characters::channel.eq(channel))
            .first(&DB.get())?)
    }

    // Attributes
    pub fn add_attribute(attr: &Attribute) -> QueryResult<Attribute> {
        Ok(diesel::insert_into(attributes::table)
            .values(attr)
            .get_result(&DB.get())?)
    }

    pub fn del_attribute(attr: &Attribute) -> QueryResult<Attribute> {
        Ok(diesel::delete(attr).get_result(&DB.get())?)
    }

    pub fn get_attribute(name: &str, pin: i64) -> QueryResult<Attribute> {
        Ok(attributes::table.find((name, pin)).first(&DB.get())?)
    }

    pub fn get_attributes(ch: &Character) -> QueryResult<Vec<Attribute>> {
        Ok(Attribute::belonging_to(ch).get_results(&DB.get())?)
    }

    pub fn update_attribute(attr: &Attribute) -> QueryResult<Attribute> {
        Ok(diesel::update(attr).set(attr).get_result(&DB.get())?)
    }

    // Notes
    pub fn del_note(note: &Note) -> QueryResult<Note> {
        Ok(diesel::delete(note).get_result(&DB.get())?)
    }

    // pub fn get_note(name: &str, pin: i64) -> QueryResult<Note> {
    //     Ok(notes::table.find((name, pin)).first(&DB.get())?)
    // }

    pub fn get_notes(ch: &Character) -> QueryResult<Vec<Note>> {
        Ok(Note::belonging_to(ch).get_results(&DB.get())?)
    }

    pub fn set_note(note: &Note) -> QueryResult<Note> {
        Ok(diesel::insert_into(notes::table)
            .values(note)
            .on_conflict((notes::pin, notes::name))
            .do_update()
            .set(note)
            .get_result(&DB.get())?)
    }

    // Channels
    pub fn add_channel(ch: &Channel) -> QueryResult<Channel> {
        Ok(diesel::insert_into(channels::table)
            .values(ch)
            .get_result(&DB.get())?)
    }

    pub fn del_channel(channel: &Channel) -> QueryResult<Channel> {
        Ok(diesel::delete(channel).get_result(&DB.get())?)
    }

    pub fn get_channel(channel: i64) -> QueryResult<Channel> {
        Ok(channels::table.find(channel).first(&DB.get())?)
    }
}

pub struct BotInfo;
impl BotInfo {
    pub fn set(bot: &Bot) -> QueryResult<Bot> {
        Ok(diesel::insert_into(bot::table)
            .values(bot)
            .on_conflict(bot::id)
            .do_update()
            .set(bot)
            .get_result(&DB.get())?)
    }

    pub fn get_activity(id: i64) -> QueryResult<Activity> {
        use ActivityKind::*;
        let bot: Bot = bot::table.find(id).first(&DB.get())?;
        match bot.activity_type {
            Playing => Ok(Activity::playing(&bot.activity_name)),
            Listening => Ok(Activity::listening(&bot.activity_name)),
            Streaming => Ok(Activity::streaming(&bot.activity_name, "https://github.com/Celti/eris")),
        }
    }
}
