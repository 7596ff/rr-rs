use sqlx::PgPool;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // load dotenv
    dotenv::dotenv()?;

    // configure sqlx
    let postgres = PgPool::connect(&dotenv::var("DATABASE_URL")?).await?;

    // run migrations
    sqlx::migrate!("./migrations/").run(&postgres).await?;

    Ok(())
}
