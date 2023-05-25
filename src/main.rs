
use {config::Config};

mod connection;
mod config;
mod models;

fn main() {
    let config_path = std::env::var("EM_CONFIG")
        .unwrap_or("config.yaml".to_string());
    println!("config path: {}", config_path);
    let config = Config::parse(&config_path)
        .expect("couldn't parse config");

    let connection = connection::imap::connect(&config)
        .expect("couldn't connect to smtp server");


    println!("{:?}", config);

}
