
use {config::Config};

mod connection;
mod config;

fn main() {
    let config = Config::parse("config.yaml");

    println!("Hello, world!");
}
