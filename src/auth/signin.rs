use std::sync::Arc;

use axum::{
    extract::State,
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::query;
use uuid::Uuid;

use crate::{
    AppState, errors::errors::AppError, helpers::passwords::hash_password,
};

#[derive(Deserialize)]
pub struct SignInPayload {
    email: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    message: String,
}

#[axum::debug_handler]
pub async fn signup(
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
