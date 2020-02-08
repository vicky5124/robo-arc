use serde::Deserialize;
use postgres::{
    Client,
    NoTls,
};
use std::{
    fs::File,
    io::prelude::*,
};

#[derive(Deserialize)]
struct Config {
    psql: Psql,
}

#[derive(Deserialize)]
struct Psql {
    username: String,
    password: String,
    database_name: String,
    host: String,
    port: String,
}

pub fn get_database() -> Result<Client, Box<dyn std::error::Error>> {
    let mut file = File::open("config.toml")?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let tokens: Config = toml::from_str(&contents.to_owned()[..]).unwrap();

    let client = Client::connect(
        &format!("host={} user={} password={} dbname={} port={}",
                 tokens.psql.host, tokens.psql.username, tokens.psql.password, tokens.psql.database_name, tokens.psql.port
        ).to_owned()[..],
        NoTls
    )?;
    Ok(client)
}
