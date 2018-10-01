#![deny(warnings)]
#![feature(macro_at_most_once_rep)]
#![feature(nll)]
#![feature(plugin)]
#![feature(tool_lints)]
#![feature(trace_macros)]
#![feature(try_blocks)]
#![plugin(phf_macros)]

#![allow(proc_macro_derive_resolution_fallback)]    // FIXME Diesel 1.4+
#[macro_use] extern crate diesel;                   // FIXME Rust 2018 / Diesel 1.4

#[macro_use] mod macros;

mod db;
mod ext;
mod framework;
mod handler;
mod model;
mod modules;

use crate::framework::Framework;
use crate::handler::Handler;
use crate::model::*;

use maplit::hashset;
use serenity::prelude::*;

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

fn main() {
    dotenv::dotenv().ok();
    log_panics::init();
    init_logger();

    let token = std::env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN");
    let mut client = Client::new(&token, Handler).expect("serenity client");
    let info = serenity::http::get_current_application_info().expect("Discord API");

    {
        let mut data = client.data.lock();
        data.insert::<SerenityShardManager>(Arc::clone(&client.shard_manager));
        data.insert::<Owner>(info.owner.id);
    }

    client.with_framework(Framework::standard(hashset!(info.owner.id)));
    client.start_autosharded().expect("serenity client");
}
