use std::env;
use sqlx::postgres::PgPool;
use darkredis::ConnectionPool;

// This function obtains a database connection to the postgresql database used for the bot.
pub async fn obtain_postgres_pool() -> Result<PgPool, Box<dyn std::error::Error>> {
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

pub async fn obtain_redis_pool() -> Result<ConnectionPool, Box<dyn std::error::Error>> {
    let redis_url = env::var("REDIS_URL")?;
    let pool = ConnectionPool::create(
        redis_url.into(),
        None,
        num_cpus::get()
    ).await?;

    Ok(pool)
}
