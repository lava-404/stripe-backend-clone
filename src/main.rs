use std::sync::Arc;
use axum::{
 Router, routing::{get, post},
};

use chrono::{Duration};
use sqlx::{PgPool};
use tokio::net::TcpListener;
use uuid::Uuid;


use crate::{auth::{jwt::{JwtService}, signin::signup}, config::config::{ApiKeyService, Config, IdempotencyStore, Metrics}};
mod auth;
mod config;
mod helpers;
mod errors;

const ACCESS_TOKEN_EXPIRY: Duration = Duration::minutes(15);
const REFRESH_TOKEN_EXPIRY: Duration = Duration::days(30);

const JWT_SECRET_KEY: &str =
    "FPvM+LeknSsMnO4lEuogtpa0aYSYqC+m2TufFlY3CZrX/DJgZiFxvHTjeKPT4woERNXXgz6vToD484Xw5WRqQg==";

const WEBHOOK_SECRET_KEY: &str =
    "lJhETBC/5bKgh6N1+J2pPYCpiN+9T2F2lUyAYtCCfgyC7jNG1Vt86/6LHJ8CT/y8B2KEseEaQWHazmBTduGQqw==";


struct AppState {
    config: Config,
    db: PgPool,
    jwt: JwtService,
    api_key: ApiKeyService,
    idempotency: IdempotencyStore,
    metrics: Metrics,
}

#[axum::debug_handler]
async fn get_api_keys() -> String {
    Uuid::new_v4().to_string()
}

#[tokio::main]
async fn main() {
    let database_url = "postgresql://neondb_owner:npg_lqSwdo0JT2Bt@ep-wild-field-ao7c4rsu-pooler.c-2.ap-southeast-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require";

    let db = PgPool::connect(database_url).await.unwrap();

    let config = Config {
        database_url: database_url.to_string(),
        jwt_secret: JWT_SECRET_KEY.to_string(),
        server_port: 3000,
        webhook_secret: WEBHOOK_SECRET_KEY.to_string(),
        frontend_url: String::from("http://localhost:3001"),
    };

    let shared_state = Arc::new(AppState {
        config,
        db: db.clone(),
        jwt: JwtService {
            secret: JWT_SECRET_KEY.to_string(),
        },
        api_key: ApiKeyService { db: db.clone() },
        idempotency: IdempotencyStore { db: db.clone() },
        metrics: Metrics {
            latency: 0,
            active_connections: 0,
            db_queries: 0,
        },
    });

    let router = Router::new()
        .route("/", get(|| async { "Hello World" }))
        .route("/signup", post(signup).with_state(shared_state));

    let listener = TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}

