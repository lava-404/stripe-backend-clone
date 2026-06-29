use sqlx::PgPool;
pub struct ApiKeyService {
  pub db: PgPool,
}

pub struct IdempotencyStore {
  pub db: PgPool,
}

pub struct Metrics {
  pub latency: u64,
  pub active_connections: u64,
  pub db_queries: u64,
}


pub struct Config {
  pub database_url: String,
  pub jwt_secret: String,
  pub server_port: u16,
  pub webhook_secret: String,
  pub frontend_url: String,
}
