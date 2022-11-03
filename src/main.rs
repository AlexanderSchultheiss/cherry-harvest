use std::env::args;
#[macro_use]
extern crate log;

fn main() {
    env_logger::init();

    info!("starting up");
}
