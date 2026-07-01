use std::{sync::Arc, time::Duration};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, query, query_as};
use tokio::time::sleep;
use uuid::Uuid;

use crate::{AppState, errors::errors::AppError};

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Price {
    pub id: Uuid,
    pub product_id: Uuid,
    pub unit_amount: i64,
    pub currency: String,
    pub recurring_interval: Option<String>,
    pub recurring_interval_count: Option<i32>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Subscription {
    pub id: Uuid,
    pub user_id: Uuid,
    pub price_id: Uuid,

    pub status: String,

    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,

    pub cancel_at_period_end: bool,

    pub next_billing_at: DateTime<Utc>,

    pub metadata: serde_json::Value,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
pub async fn process_due_subscriptions(
  state: Arc<AppState>,
) -> Result<(), AppError> {
  let subscriptions = query_as::<_, Subscription>(
      r#"
      SELECT *
      FROM subscriptions
      WHERE next_billing_at <= NOW()
        AND status = 'active'
      "#,
  )
  .fetch_all(&state.db)
  .await
  .map_err(|e| {
      eprintln!("Database error: {:#?}", e);
      AppError::DatabaseError
  })?;

  for subscription in subscriptions {
    println!("Processing subscription {}", subscription.id);

    // Fetch the associated price.
    let price: Price = query_as(
      r#"
      SELECT *
      FROM prices
      WHERE id = $1
      "#,
  )
  .bind(subscription.price_id)
  .fetch_one(&state.db)
  .await
  .map_err(|_| AppError::DatabaseError)?;

    // Create a new Payment Intent.
    let payment_intent_id = Uuid::new_v4();
    let client_secret = Uuid::new_v4().to_string();
    let now = Utc::now();

    query(
        r#"
        INSERT INTO payment_intents (
            id,
            user_id,
            price_id,
            amount,
            currency,
            status,
            client_secret,
            created_at,
            updated_at
        )
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
        "#,
    )
    .bind(payment_intent_id)
    .bind(subscription.user_id)
    .bind(subscription.price_id)
    .bind(price.unit_amount)
    .bind(price.currency.clone())
    .bind("processing")
    .bind(client_secret)
    .bind(now)
    .bind(now)
    .execute(&state.db)
    .await
    .map_err(|_| AppError::DatabaseError)?;

    // Simulate attempting the payment.
    // Later this is where you'll contact a payment processor.
    let payment_succeeded = true;

    if payment_succeeded {
        // Compute the next billing period.
        let next_period_start = subscription.current_period_end;
        let next_period_end = next_period_start + chrono::Duration::days(30);

        // Update the subscription.
        query(
            r#"
            UPDATE subscriptions
            SET
                current_period_start = $1,
                current_period_end = $2,
                next_billing_at = $2,
                updated_at = $3
            WHERE id = $4
            "#,
        )
        .bind(next_period_start)
        .bind(next_period_end)
        .bind(Utc::now())
        .bind(subscription.id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?;

        // Mark the Payment Intent as succeeded.
        query(
            r#"
            UPDATE payment_intents
            SET
                status = 'succeeded',
                amount_received = amount,
                updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(payment_intent_id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?;
    } else {
        // Mark the Payment Intent as failed.
        query(
            r#"
            UPDATE payment_intents
            SET
                status = 'failed',
                updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(payment_intent_id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?;

        // Mark the subscription as past_due.
        query(
            r#"
            UPDATE subscriptions
            SET
                status = 'past_due',
                updated_at = $1
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(subscription.id)
        .execute(&state.db)
        .await
        .map_err(|_| AppError::DatabaseError)?;

        // TODO:
        // Schedule a retry. This could be done by:
        // - setting next_billing_at = NOW() + INTERVAL '1 day'
        // - incrementing a retry_count
        // - enqueueing a background job
    }
}

  Ok(())
}


pub async fn subscription_worker(state: Arc<AppState>) {
  loop {
      if let Err(e) = process_due_subscriptions(state.clone()).await {
          eprintln!("Subscription worker error: {:?}", e);
      }

      sleep(Duration::from_secs(60)).await;
  }
}
