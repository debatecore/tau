use std::time::Duration;

use axum::Router;
use reqwest::Client;
use sqlx::{postgres::PgPoolOptions, PgPool};
use testcontainers_modules::{
    postgres::Postgres,
    testcontainers::{runners::AsyncRunner, ContainerAsync},
};
use tokio::net::TcpListener;
use tau::{
    database,
    routes,
    setup::{self, AppState},
    users::infradmin::guarantee_infrastructure_admin_exists,
};
use tower_cookies::CookieManagerLayer;
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub client: Client,
    pub pool: PgPool,
    pub db_name: String,
    _db_container: ContainerAsync<Postgres>,
}

impl TestApp {
    pub async fn spawn() -> Self {
        setup::read_environmental_variables();
        ensure_test_env();

        let db_name = format!("test_{}", Uuid::now_v7().simple());

        let db_container = Postgres::default()
            .with_db_name(&db_name)
            .with_user("postgres")
            .with_password("postgres")
            .start()
            .await
            .expect("failed to start postgres container; is Docker running and is /var/run/docker.sock available?");

        let host = db_container
            .get_host()
            .await
            .expect("failed to get container host");

        let port = db_container
            .get_host_port_ipv4(5432)
            .await
            .expect("failed to get mapped postgres port");

        let database_url = format!(
            "postgres://postgres:postgres@{}:{}/{}",
            host, port, db_name
        );

        let pool = PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&database_url)
            .await
            .expect("failed to connect to postgres in container");

        prepare_empty_database(&pool).await;

        let state = create_test_app_state(pool.clone());
        let app = create_test_app(state);

        let listener = create_test_listener().await;
        let local_addr = listener.local_addr().unwrap();
        let address = format!("http://{}", local_addr);

        tokio::spawn(async move {
            axum::serve(listener, app)
                .await
                .expect("test server crashed");
        });

        let client = Client::builder()
            .cookie_store(true)
            .build()
            .expect("failed to build reqwest client");

        Self {
            address,
            client,
            pool,
            db_name,
            _db_container: db_container,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.address, path)
    }
}

fn create_test_app_state(pool: PgPool) -> AppState {
    setup::create_app_state_with_pool(pool)
}

fn create_test_app(state: AppState) -> Router {
    Router::new()
        .merge(routes::routes())
        .with_state(state)
        .layer(setup::configure_cors())
        .layer(CookieManagerLayer::new())
}

async fn create_test_listener() -> TcpListener {
    TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind random port")
}

async fn prepare_empty_database(pool: &PgPool) {
    database::clear_database(pool).await;
    database::perform_migrations(pool).await;
    guarantee_infrastructure_admin_exists(pool).await;
}

fn ensure_test_env() {
    if std::env::var("SECRET").is_err() {
        std::env::set_var("SECRET", "test-secret");
    }

    if std::env::var("FRONTEND_ORIGIN").is_err() {
        std::env::set_var("FRONTEND_ORIGIN", "http://localhost:3000");
    }
}