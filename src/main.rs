#![recursion_limit = "1024"]
#![feature(match_default_bindings)]
#![feature(nll)]
#![feature(try_trait)]
#![feature(use_nested_groups)]

#[macro_use] extern crate failure;
#[macro_use] extern crate indoc;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;
#[macro_use] extern crate maplit;
#[macro_use] extern crate matches;
#[macro_use] extern crate serenity;

extern crate chrono;
extern crate ddate;
extern crate env_logger;
extern crate fnorder;
extern crate log_panics;
extern crate parking_lot;
extern crate rand;
extern crate regex;
extern crate rink;
extern crate typemap;

mod commands;
mod data;
mod eris;
mod ext;
mod utils;

fn main() {
    log_panics::init();
    env_logger::init();

    if let Err(err) = eris::run() {
        let mut causes = err.causes();
        error!("Error: {}", causes.next().unwrap()); // Causes always contains at least one Fail.
        for cause in causes {
            error!("Caused by: {}", cause);
        }
        error!("{}", err.backtrace());
        std::process::exit(1);
    }
}
