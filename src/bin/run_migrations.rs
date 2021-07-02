use sqlx::PgPool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // load dotenv
    dotenv::dotenv()?;

    // configure sqlx
    let postgres = PgPool::connect(&dotenv::var("DATABASE_URL")?).await?;

    // run migrations
    sqlx::migrate!("./migrations/").run(&postgres).await?;

    Ok(())
}
