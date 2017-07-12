#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate serenity;
#[macro_use]
extern crate lazy_static;

extern crate fnorder;
extern crate rand;
extern crate regex;
extern crate rink;

mod errors;
mod commands;

use errors::*;

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        use error_chain::ChainedError; // trait which holds `display`
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "{}", e.display()).expect(errmsg);
        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    use serenity::Client;
    use std::env;

    let mut client =
        Client::new(&env::var("DISCORD_TOKEN").chain_err(|| "could not get Discord token.")?);

    client.with_framework(|f| {
        f.configure(|c| c.prefix("."))
            .command("ping",  |c| c.exec(commands::meta::ping))
            .command("foo",   |c| c.exec(commands::meta::foo))
            .command("roll",  |c| c.exec(commands::dice::roll))
            .command("calc",  |c| c.exec(commands::calc::calc))
            .command("st",    |c| c.exec(commands::gurps::st))
            .command("fnord", |c| c.exec(commands::toys::fnord))
    });

    client.start().chain_err(|| "failed to start shard")
}
