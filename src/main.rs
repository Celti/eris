#![deny(rust_2018_idioms)]

#[macro_use]
extern crate diesel;
#[macro_use]
mod macros;

mod db;
mod ext;
mod framework;
mod handler;
mod model;
mod modules;

use crate::framework::Framework;
use crate::handler::Handler;
use crate::model::*;
use serenity::prelude::*;
use std::error::Error;
use std::sync::Arc;

fn init_logger() {
    if std::env::var("RUST_LOG_TARGET").ok().as_deref() == Some("systemd") {
        systemd::journal::JournalLog::init().expect("systemd journal");
        log::set_max_level(log::LevelFilter::Info);
    } else {
        pretty_env_logger::init();
    }

    log::info!("Initialised logger.");
}

fn main() -> Result<(), Box<dyn Error>> {
    dotenv::dotenv().ok();
    log_panics::init();
    init_logger();

    let token = std::env::var("DISCORD_TOKEN")?;
    let mut client = Client::new(&token, Handler)?;

    let http = client.cache_and_http.http.clone();
    let info = http.get_current_application_info()?;

    {
        let mut data = client.data.write();
        data.insert::<SerenityShardManager>(Arc::clone(&client.shard_manager));
        data.insert::<Owner>(info.owner.id);
    }

    client.with_framework(Framework::standard(info.owner.id, info.id));
    client.start_autosharded()?;

    Ok(())
}
