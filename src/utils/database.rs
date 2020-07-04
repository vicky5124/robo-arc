use std::env;
use sqlx::postgres::PgPool;

// This function obtains a database connection to the postgresql database used for the bot.
pub async fn obtain_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
    // Obtain the postgresql url.
    let pg_url = env::var("DATABASE_URL2")?;

    // Connect to the database with the information provided on the configuration.
    // and return a pool of connections
    let pool = PgPool::builder()
        .max_size(20)
        .build(&pg_url)
        .await?;

    // return the pool
    Ok(pool)
}
