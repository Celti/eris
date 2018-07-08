#![feature(nll)]
#![feature(option_filter)]

extern crate chrono;
extern crate ddate;
#[macro_use] extern crate diesel;
extern crate dotenv;
extern crate env_logger;
extern crate fnorder;
#[macro_use] extern crate indoc;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
extern crate log_panics;
#[macro_use] extern crate maplit;
#[macro_use] extern crate matches;
extern crate parking_lot;
extern crate radix_trie;
extern crate rand;
extern crate regex;
#[macro_use] extern crate serenity;
extern crate typemap;

mod db;
mod schema;

mod key;
mod ext;
mod utils;

mod eris;
mod commands;

fn main() {
    dotenv::dotenv().ok();
    log_panics::init();
    env_logger::init();

    if let Err(err) = eris::run() {
        error!("Error: {}", err);
        std::process::exit(1);
    }
}
