use std::sync::Arc;

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, query};
use tokio::net::TcpListener;
use uuid::Uuid;

// ─────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────

const ACCESS_TOKEN_EXPIRY: Duration = Duration::minutes(15);
const REFRESH_TOKEN_EXPIRY: Duration = Duration::days(30);

const JWT_SECRET_KEY: &str =
    "FPvM+LeknSsMnO4lEuogtpa0aYSYqC+m2TufFlY3CZrX/DJgZiFxvHTjeKPT4woERNXXgz6vToD484Xw5WRqQg==";

const WEBHOOK_SECRET_KEY: &str =
    "lJhETBC/5bKgh6N1+J2pPYCpiN+9T2F2lUyAYtCCfgyC7jNG1Vt86/6LHJ8CT/y8B2KEseEaQWHazmBTduGQqw==";

// ─────────────────────────────────────────────
// Config
// ─────────────────────────────────────────────

struct Config {
    database_url: String,
    jwt_secret: String,
    server_port: u16,
    webhook_secret: String,
    frontend_url: String,
}

// ─────────────────────────────────────────────
// Error types
// ─────────────────────────────────────────────

#[derive(Debug)]
pub enum AppError {
    UserNotFound,
    InvalidCredentials,
    EmailAlreadyExists,
    DatabaseError,
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::UserNotFound => (StatusCode::NOT_FOUND, "User does not exist"),
            AppError::InvalidCredentials => (StatusCode::UNAUTHORIZED, "Invalid email or password"),
            AppError::EmailAlreadyExists => (StatusCode::CONFLICT, "Email already exists"),
            AppError::DatabaseError => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        (
            status,
            Json(ErrorResponse {
                message: message.to_string(),
            }),
        )
            .into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum JwtError {
    #[error("invalid token")]
    InvalidToken,

    #[error("token expired")]
    Expired,

    #[error("failed to create token")]
    CreationFailed,

    #[error("failed to create token")]
    Jwt(jsonwebtoken::errors::Error),
}

// ─────────────────────────────────────────────
// JWT
// ─────────────────────────────────────────────

#[derive(Serialize, Deserialize)]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub token_type: TokenType,
    pub exp: usize,
    pub iat: usize,
}

struct JwtService {
    secret: String,
}

impl JwtService {
    fn create_token(&self, user_id: String, token_type: TokenType, expires_in: usize) -> String {
        let now = Utc::now().timestamp() as usize;
        let claims = Claims {
            sub: user_id,
            token_type,
            exp: expires_in,
            iat: now,
        };

        encode(
            &Header::new(Algorithm::HS256),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .unwrap()
    }

    pub fn create_access_token(&self, user_id: Uuid) -> String {
        self.create_token(
            user_id.to_string(),
            TokenType::Access,
            (Utc::now() + ACCESS_TOKEN_EXPIRY).timestamp() as usize,
        )
    }

    pub fn create_refresh_token(&self, user_id: Uuid) -> String {
        self.create_token(
            user_id.to_string(),
            TokenType::Refresh,
            (Utc::now() + REFRESH_TOKEN_EXPIRY).timestamp() as usize,
        )
    }

    pub fn decode_token(&self, token: String) -> Result<(Claims, String), JwtError> {
        let token_data = decode::<Claims>(
            &token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(JwtError::Jwt)?;

        let token_type = match token_data.claims.token_type {
            TokenType::Access => "access".to_string(),
            TokenType::Refresh => "refresh".to_string(),
        };

        Ok((token_data.claims, token_type))
    }
}

// ─────────────────────────────────────────────
// Password helpers
// ─────────────────────────────────────────────

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(password_hash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

// ─────────────────────────────────────────────
// Supporting services
// ─────────────────────────────────────────────

pub struct WebhookSigner {
    secret: String,
}

pub struct ApiKeyService {
    db: PgPool,
}

pub struct IdempotencyStore {
    db: PgPool,
}

pub struct Metrics {
    latency: u64,
    active_connections: u64,
    db_queries: u64,
}

// ─────────────────────────────────────────────
// Application state
// ─────────────────────────────────────────────

struct AppState {
    config: Config,
    db: PgPool,
    jwt: JwtService,
    api_key: ApiKeyService,
    idempotency: IdempotencyStore,
    metrics: Metrics,
}

// ─────────────────────────────────────────────
// Request / response types
// ─────────────────────────────────────────────

#[derive(Deserialize)]
struct SignInPayload {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    message: String,
}

pub struct SignInResponse {
    message: String,
    access_token: String,
    refresh_token: String,
}

// ─────────────────────────────────────────────
// Handlers
// ─────────────────────────────────────────────

#[axum::debug_handler]
async fn get_api_keys() -> String {
    Uuid::new_v4().to_string()
}

#[axum::debug_handler]
async fn signup(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(payload): Json<SignInPayload>,
) -> Result<impl IntoResponse, AppError> {
    println!("Reached signup");
    // Check whether the user already exists.
    let user_exists = query("SELECT * FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| {
            println!("Database error: {e}");
            AppError::DatabaseError
        })?;
    println!("Checked if user exists");
    if let Some(_user) = user_exists {
        return Err(AppError::EmailAlreadyExists);
    }

    // Hash the password.
    let password_hash =
        hash_password(payload.password.as_str()).map_err(|_| AppError::InvalidCredentials)?;
    println!("Password hashed");

    // Insert the new user.
    let id = Uuid::new_v4();
    let now = Utc::now();

    query(
        r#"
        INSERT INTO users (id, email, password_hash, created_at, updated_at)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, email
        "#,
    )
    .bind(id)
    .bind(payload.email)
    .bind(password_hash)
    .bind(now)
    .bind(now)
    .fetch_one(&state.db)
    .await
    .map_err(|e| {
        eprintln!("Insert error: {:#?}", e);
        AppError::DatabaseError
    })?;
    println!("Inserted user");

    // Issue tokens.
    let access_token = state.jwt.create_access_token(id);
    let refresh_token = state.jwt.create_refresh_token(id);

    // Set cookies.
    let jar = jar
        .add(
            Cookie::build(("access_token", access_token))
                .http_only(true)
                .secure(false)
                .path("/")
                .build(),
        )
        .add(
            Cookie::build(("refresh_token", refresh_token))
                .http_only(true)
                .secure(false)
                .path("/")
                .build(),
        );

    Ok((
        jar,
        Json(LoginResponse {
            message: "Login successful".into(),
        }),
    ))
}

// ─────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────

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

