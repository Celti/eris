pub mod model;
mod schema;

use crate::db::model::*;
use crate::db::schema::*;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    pub static ref DB: Database = Database::connect();
}

no_arg_sql_function!(random, (), "Represents the SQL RANDOM() function");

pub struct Database {
    pool: Pool<ConnectionManager<PgConnection>>,
}

impl Database {
    pub fn connect() -> Self {
        let url = std::env::var("DATABASE_URL").expect("DATABASE_URL");
        let manager = ConnectionManager::<PgConnection>::new(url);
        let pool = Pool::new(manager).expect("database connection pool");

        Self { pool }
    }

    fn get(&self) -> PooledConnection<ConnectionManager<PgConnection>> {
        self.pool.clone().get().expect("database connection")
    }

    pub fn get_prefixes(&self) -> QueryResult<HashMap<i64, String>> {
        Ok(prefixes::table.load(&self.get())?.into_iter().collect())
    }

    pub fn upsert_prefix(&self, prefix: &Prefix) -> QueryResult<Prefix> {
        Ok(diesel::insert_into(prefixes::table)
            .values(prefix)
            .on_conflict(prefixes::id)
            .do_update()
            .set(prefix)
            .get_result(&self.get())?)
    }

    pub fn get_keyword(&self, kw: &str) -> QueryResult<Keyword> {
        Ok(keywords::table.find(kw).first(&self.get())?)
    }

    pub fn add_keyword(&self, kw: &Keyword) -> QueryResult<Keyword> {
        Ok(diesel::insert_into(keywords::table)
            .values(kw)
            .get_result(&self.get())?)
    }

    pub fn update_keyword(&self, kw: &Keyword) -> QueryResult<Keyword> {
        Ok(diesel::update(kw).set(kw).get_result(&self.get())?)
        // Ok(kw.save_changes(&self.get())?)
    }

    // pub fn del_keyword(&self, kw: &Keyword) -> QueryResult<Keyword> {
    //     Ok(diesel::delete(kw).get_result(&self.get())?)
    // }

    pub fn find_keywords(&self, partial: &str) -> QueryResult<Vec<Keyword>> {
        let find = format!("%{}%", partial);
        Ok(keywords::table
            .filter(keywords::keyword.ilike(&find))
            .get_results(&self.get())?)
    }

    pub fn get_bareword(&self, word: &str) -> QueryResult<Definition> {
        let kw = keywords::table
            .find(word)
            .filter(keywords::bareword.eq(true))
            .first::<Keyword>(&self.get())?;

        Ok(Definition::belonging_to(&kw)
            .order_by(random)
            .first(&self.get())?)
    }

    pub fn get_definition(&self, kw: &str, def: &str) -> QueryResult<Definition> {
        Ok(definitions::table.find((kw, def)).first(&self.get())?)
    }

    pub fn get_definitions(&self, kw: &Keyword) -> QueryResult<Vec<Definition>> {
        Ok(Definition::belonging_to(kw).get_results(&self.get())?)
    }

    pub fn add_definition(&self, def: &Definition) -> QueryResult<Definition> {
        Ok(diesel::insert_into(definitions::table)
            .values(def)
            .get_result(&self.get())?)
    }

    pub fn del_definition(&self, def: &Definition) -> QueryResult<Definition> {
        Ok(diesel::delete(def).get_result(&self.get())?)
    }

    pub fn find_definitions(&self, kw: &Keyword, partial: &str) -> QueryResult<Vec<Definition>> {
        let find = format!("%{}%", partial);
        Ok(Definition::belonging_to(kw)
            .filter(definitions::definition.ilike(&find))
            .get_results(&self.get())?)
    }

    pub fn add_character(&self, ch: &Character) -> QueryResult<Character> {
        Ok(diesel::insert_into(characters::table)
            .values(ch)
            .get_result(&self.get())?)
    }

    pub fn add_attribute(&self, attr: &Attribute) -> QueryResult<Attribute> {
        Ok(diesel::insert_into(attributes::table)
            .values(attr)
            .get_result(&self.get())?)
    }

    pub fn del_character(&self, ch: &Character) -> QueryResult<Character> {
        Ok(diesel::delete(ch).get_result(&self.get())?)
    }

    pub fn del_attribute(&self, attr: &Attribute) -> QueryResult<Attribute> {
        Ok(diesel::delete(attr).get_result(&self.get())?)
    }

    pub fn get_character_by_pair(&self, name: &str, channel: i64) -> QueryResult<Character> {
        Ok(characters::table
            .filter(characters::name.eq(name))
            .filter(characters::channel.eq(channel))
            .first(&self.get())?)
    }

    // pub fn get_character_by_pin(&self, pin: i64) -> QueryResult<Character> {
    //     Ok(characters::table
    //         .filter(characters::pin.eq(pin))
    //         .first(&self.get())?)
    // }

    pub fn get_attributes(&self, ch: &Character) -> QueryResult<Vec<Attribute>> {
        Ok(Attribute::belonging_to(ch).get_results(&self.get())?)
    }

    pub fn get_attribute_by_pair(&self, name: &str, pin: i64) -> QueryResult<Attribute> {
        Ok(attributes::table.find((name, pin)).first(&self.get())?)
    }

    pub fn update_attribute(&self, attr: &Attribute) -> QueryResult<Attribute> {
        Ok(diesel::update(attr).set(attr).get_result(&self.get())?)
    }

    pub fn del_channel(&self, channel: &Channel) -> QueryResult<Channel> {
        Ok(diesel::delete(channel).get_result(&self.get())?)
    }

    pub fn get_channel(&self, channel: i64) -> QueryResult<Channel> {
        Ok(channels::table.find(channel).first(&self.get())?)
    }

    pub fn set_channel(&self, ch: &Channel) -> QueryResult<Channel> {
        Ok(diesel::insert_into(channels::table)
            .values(ch)
            .get_result(&self.get())?)
    }
}
