use axum::http::{header::CONTENT_TYPE, HeaderValue, Method};
use sqlx::{Pool, Postgres};
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

use crate::database;

const CRYPTO_SECRET_CORRECT: &str = "Cryptographic SECRET is set.";
const CRYPTO_SECRET_NOT_SET: &str = "Cryptographic SECRET is not set. This may lead to increased predictability in token generation.";
const CRYPTO_SECRET_ERROR: &str = "Could not read SECRET. Is it valid UTF-8?";
const FRONTEND_ORIGIN_NOT_SET: &str = "FRONTEND_ORIGIN is not set. Please provide a valid URL leading to an accepted origin.";

pub fn initialise_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("setting default tracing subscriber failed!");
    info!("Response cannon spinning up...");
}

pub fn report_listener_socket_addr(listener: &TcpListener) {
    let addr = match listener.local_addr() {
        Ok(addr) => addr,
        Err(e) => {
            error!("Error getting listener socket address: {e}");
            panic!();
        }
    };
    info!("Listener socket address is: {}", addr.to_string());
}

fn get_env_port() -> u16 {
    let portstr = match std::env::var("PORT") {
        Ok(value) => match value.is_empty() {
            true => return 2023,
            false => value,
        },
        Err(_) => return 2023,
    };

    return match portstr.parse() {
        Ok(num) => num,
        Err(e) => {
            error!("Error parsing PORT environment variable: {e}");
            panic!();
        }
    };
}

pub fn get_socket_addr() -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), get_env_port())
}

pub fn get_local_socket_addr() -> SocketAddrV4 {
    SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), get_env_port())
}

#[derive(Clone)]
pub struct AppState {
    pub connection_pool: Pool<Postgres>,
}

pub async fn create_app_state() -> AppState {
    AppState {
        connection_pool: database::get_connection_pool().await,
    }
}

pub fn read_environmental_variables() {
    match dotenvy::dotenv() {
        Ok(_) => info!("Loaded .env"),
        Err(e) => {
            if e.not_found() {
                warn!("No .env file found; skipping...");
            } else {
                error!("Error loading .env file!: {e}");
                panic!();
            }
        }
    }
}

pub fn check_secret_env_var() {
    match std::env::var("SECRET") {
        Ok(v) => match v.is_empty() {
            true => warn!("{}", CRYPTO_SECRET_NOT_SET),
            false => info!("{}", CRYPTO_SECRET_CORRECT),
        },
        Err(e) => match e {
            std::env::VarError::NotPresent => {
                warn!("{}", CRYPTO_SECRET_NOT_SET);
            }
            _ => {
                error!("{}", CRYPTO_SECRET_ERROR);
                panic!();
            }
        },
    }
}

pub fn configure_cors() -> CorsLayer {
    let default_origin = "http://localhost:3000".to_owned();
    let result = std::env::var("FRONTEND_ORIGIN");

    #[cfg(not(debug_assertions))]
    if result.is_err() {
        error!("{}", FRONTEND_ORIGIN_NOT_SET);
        panic!();
    }

    let frontend_origin = result.unwrap_or(default_origin);
    info!(
        "FRONTEND_ORIGIN set to {}. Requests made from any other origins will be disallowed at browser level",
        &frontend_origin
    );
    let layer = CorsLayer::new()
        .allow_origin(frontend_origin.parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::PATCH,
            Method::PUT,
        ])
        .allow_headers([CONTENT_TYPE])
        .allow_credentials(true);

    return layer;
}
