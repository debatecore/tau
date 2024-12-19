use app_state::{AppState, AppStateTrait};
use sqlx::{Pool, Postgres};
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio::net::TcpListener;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use crate::database;

pub fn initialise_logging() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
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
        Ok(value) => value,
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

pub struct TauState {
    pub connection_pool: Pool<Postgres>,
}

pub async fn create_app_state() {
    AppState::init(TauState {
        connection_pool: database::get_connection_pool().await,
    });
}

pub fn read_environmental_variables() {
    match dotenvy::dotenv() {
        Ok(_) => info!("Loaded .env"),
        Err(e) => {
            error!("Error reading .env file: {e}");
            panic!();
        }
    }
}
