use sqlx::{postgres::PgConnectOptions, Pool, Postgres};
use std::env;
use tracing::{error, info};

pub async fn get_connection_pool() -> Pool<Postgres> {
    info!("Attempting to connect to a database…");
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
    let url = env::var("DATABASE_URL").expect("DATABASE_URL must be defined in .env");
    let username =
        env::var("DATABASE_USERNAME").expect("DATABASE_USERNAME must be defined in .env");
    let password =
        env::var("DATABASE_PASSWORD").expect("DATABASE_PASSWORD must be defined in .env");
    let port_number = get_port_number();

    let options = PgConnectOptions::new()
        .host(&url)
        .port(port_number)
        .username(&username)
        .password(&password);

    Pool::connect_with(options).await
}

fn get_port_number() -> u16 {
    let port_number = env::var("DATABASE_PORT_NUMBER")
        .expect("DATABASE_PORT_NUMBER must be defined in .env");
    port_number
        .parse::<u16>()
        .expect("Port number must be a number")
}
