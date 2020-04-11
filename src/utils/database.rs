use std::{
    fs::File,
    io::prelude::*,
};

use serde::Deserialize;
use sqlx::postgres::PgPool;

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
pub async fn obtain_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    // Open the configuration file
    let mut file = File::open("config.toml")?;
    // and read it's content into a String
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;

    //Serialize the data into the structures. 
    let tokens: Config = toml::from_str(&contents.as_str()).unwrap();

    // Build the postgresql url.
    let pg_url = format!("postgres://{}:{}@{}:{}/{}", tokens.psql.username, tokens.psql.password, tokens.psql.host, tokens.psql.port, tokens.psql.database_name);

    // Connect to the database with the information provided on the configuration.
    // and return a pool of connections
    let pool = PgPool::builder()
        .max_size(20)
        .build(&pg_url)
        .await?;

    // return the pool
    Ok(pool)
}
