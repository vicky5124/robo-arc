use serde::Deserialize;
use postgres::{
    Client,
    NoTls,
};
use std::{
    fs::File,
    io::prelude::*,
};

// Structs to deserialize the "config.toml" data into.
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

// This function obtains a database connection to the postgresql database used for the bot.
pub fn get_database() -> Result<Client, Box<dyn std::error::Error>> {
    // Open the configuration file
    let mut file = File::open("config.toml")?;
    // and read it's content into a String
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    //Serialize the data into the structures. 
    let tokens: Config = toml::from_str(&contents.as_str()).unwrap();

    // Connect to the database with the information provided on the configuration.
    let client = Client::connect(
        &format!("host={} user={} password={} dbname={} port={}",
                 tokens.psql.host, tokens.psql.username, tokens.psql.password, tokens.psql.database_name, tokens.psql.port
        ).to_owned()[..],
        // no Tls because the db is not ssl
        NoTls
    )?;
    // return the client connection
    Ok(client)
}
