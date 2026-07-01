use std::sync::Arc;

use axum::{Json, extract::{FromRef, FromRequestParts, State}, http::{StatusCode, request::Parts}, response::IntoResponse};
use axum_extra::extract::CookieJar;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};
use uuid::Uuid;

use crate::{AppState, errors::errors::AppError};


#[derive(Debug, Serialize, FromRow)]
pub struct AuthenticatedUser {
  pub id: Uuid,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Product {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub active: bool,
    pub image_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize)]
pub struct CreateProductPayload {
    pub name: String,
    pub description: Option<String>,
    pub active: Option<bool>,
    pub image_url: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl<S> FromRequestParts<S> for AuthenticatedUser
where
    Arc<AppState>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        // Get AppState
        let state = Arc::<AppState>::from_ref(state);

        // Read cookies
        let jar = CookieJar::from_headers(&parts.headers);

        // Get the access token cookie
        let access_token = jar
            .get("access_token")
            .ok_or(AppError::InvalidCredentials)?
            .value()
            .to_string();

        // Decode the JWT
        let (claims, token_type) = state
            .jwt
            .decode_token(access_token)
            .map_err(|_| AppError::InvalidCredentials)?;

        // Ensure it's an access token
        if token_type != "access" {
            return Err(AppError::InvalidCredentials);
        }

        // Parse the UUID stored in `sub`
        let user_id = Uuid::parse_str(&claims.sub)
            .map_err(|_| AppError::InvalidCredentials)?;

        Ok(AuthenticatedUser {
            id: user_id,
        })
    }
}


//get products
#[axum::debug_handler]
pub async fn get_products( State(state): State<Arc<AppState>>,  user: AuthenticatedUser) -> Result<impl IntoResponse, AppError> {
    let products: Vec<Product> = query_as(
        r#"
        SELECT *
        FROM products
        WHERE user_id = $1
        "#,
    )
    .bind(user.id)
    .fetch_all(&state.db)
    .await
    .map_err(|_| AppError::DatabaseError)?;
    
    Ok(Json(products))
}


//create products
#[axum::debug_handler]
pub async fn create_products(
    State(state): State<Arc<AppState>>,
    user: AuthenticatedUser,
    Json(payload): Json<CreateProductPayload>,
) -> Result<impl IntoResponse, AppError> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    query(
      r#"
      INSERT INTO products (
          id,
          user_id,
          name,
          description,
          active,
          image_url,
          metadata,
          created_at,
          updated_at
      )
      VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      "#,
  )
  .bind(id)
  .bind(user.id)
  .bind(payload.name)
  .bind(payload.description)
  .bind(payload.active.unwrap_or(true))
  .bind(payload.image_url)
  .bind(payload.metadata)
  .bind(now)
  .bind(now)
  .execute(&state.db)
  .await
  .map_err(|e| {
      eprintln!("Create product error: {:#?}", e);
      AppError::DatabaseError
  })?;

    Ok(StatusCode::CREATED)
}