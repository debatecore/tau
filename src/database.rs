use sqlx::{migrate, postgres::PgConnectOptions, query, Pool, Postgres};
use std::{env, str::FromStr};
use tracing::{error, info};

pub async fn get_connection_pool() -> Pool<Postgres> {
    info!("Attempting to connect to a database...");
    let connection_pool = match connect_to_database().await {
        Ok(connection_pool) => connection_pool,
        Err(e) => {
            error!("Error connecting to the database: {e}");
            panic!();
        }
    };
    info!("Connection with the database successful");
    connection_pool
}

async fn connect_to_database() -> Result<Pool<Postgres>, sqlx::Error> {
    let connection_uri =
        env::var("DATABASE_URL").expect("DATABASE_URL must be defined in .env");

    let options =
        PgConnectOptions::from_str(&connection_uri).expect("Connection URI is invalid");

    Pool::connect_with(options).await
}

pub async fn perform_migrations(pool: &Pool<Postgres>) {
    let result = migrate!("./migrations").run(pool).await;
    match result {
        Ok(_) => info!("Database migrations successful."),
        Err(e) => {
            error!("Error performing database migrations: {e}");
            panic!();
        }
    }
}

pub async fn clear_database(pool: &Pool<Postgres>) {
    let result = query!(
        "SELECT table_name
FROM information_schema.tables
WHERE table_schema = 'public';
"
    )
    .fetch_all(pool)
    .await
    .unwrap();
    for table in result {
        let table_name = table.table_name.unwrap();
        let drop = format!("DROP TABLE IF EXISTS {} CASCADE", table_name);
        if query(&drop).execute(pool).await.is_err() {
            panic!("Failed to drop table {}", table_name)
        }
    }
}
