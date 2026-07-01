# Stripe Clone 🚀

[![Rust](https://img.shields.io/badge/Rust-000000?style=flat\&logo=rust\&logoColor=white)](https://www.rust-lang.org/)
[![Tokio](https://img.shields.io/badge/Tokio-Async_Runtime-green)](https://tokio.rs/)
[![Axum](https://img.shields.io/badge/Axum-Web_Framework-blue)](https://github.com/tokio-rs/axum)
[![PostgreSQL](https://img.shields.io/badge/PostgreSQL-4169E1?style=flat\&logo=postgresql\&logoColor=white)](https://www.postgresql.org/)
[![SQLx](https://img.shields.io/badge/SQLx-Async_Database-orange)](https://github.com/launchbadge/sqlx)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Stripe-inspired payment infrastructure built entirely in Rust.

This project recreates many of the core backend systems behind modern payment processors, including authentication, product management, recurring pricing, subscriptions, payment intents, and automatic billing.

Rather than acting as a wrapper around Stripe's APIs, the goal is to understand and implement the internal architecture that powers subscription-based payment platforms.

---

# What It Does

The backend models the lifecycle of a payment platform from merchant authentication to recurring billing.

Current functionality includes:

* Secure JWT authentication
* Cookie-based session management
* Product management
* Multiple prices per product
* Subscription management
* Automatic recurring billing
* Background subscription workers
* Payment Intent creation
* PostgreSQL persistence

Instead of requiring merchants to manually bill customers every month, subscriptions are monitored by a background worker which automatically creates payment intents whenever a subscription reaches its next billing date.

---

Most tutorials demonstrate how to integrate Stripe into an application.

Very few explain how Stripe itself could be implemented.

This project focuses on recreating the backend architecture behind payment processors to better understand:

* Authentication systems
* Subscription billing
* Asynchronous background workers
* REST API design
* Database modeling
* Production backend architecture
* Building scalable services with Rust

---

# Features

## Authentication

Authentication is fully JWT-based.

Upon successful signup, the backend generates both an access token and a refresh token. These are returned as **HttpOnly cookies**, preventing JavaScript access and improving security.

Current authentication features include:

* User registration
* JWT generation
* Access tokens
* Refresh tokens
* Cookie-based authentication
* Protected routes
* Argon2 password hashing

Passwords are never stored in plaintext. Every password is hashed using Argon2 before being persisted to PostgreSQL.

Each JWT contains:

* User ID
* Token Type
* Issued At
* Expiration Time

Protected endpoints automatically validate the access token before allowing access.

<img width="1154" height="602" alt="image" src="https://github.com/user-attachments/assets/fc97bbc7-0e36-4580-a136-325339ae28c4" />
<img width="928" height="368" alt="image" src="https://github.com/user-attachments/assets/548243e6-c35d-4bd8-b9c6-4105a2e4af06" />


---

## Products

Products represent items or services offered by a merchant.

Each authenticated user owns their own collection of products, ensuring complete data isolation between accounts.

Current product functionality includes:

* Create products
* Retrieve products
* Ownership validation
* Product metadata
* Product activation status

Products intentionally do not contain pricing information.

Instead, pricing is handled separately, following Stripe's architecture.
<img width="1150" height="680" alt="image" src="https://github.com/user-attachments/assets/957093eb-8513-45a6-a807-f2288083ca6c" />


---

## Prices

A Product defines **what** is being sold.

A Price defines **how much** it costs.

Separating the two allows a single product to support multiple billing options.

For example, one product may offer:

* Monthly subscription
* Quarterly subscription
* Annual subscription

Current price fields include:

* Unit amount
* Currency
* Billing interval
* Billing interval count
* Active status

This design avoids duplicating product information for every pricing option.

---

## Subscriptions

Subscriptions connect customers to prices.

Rather than storing payment information directly, subscriptions reference a specific price.

Each subscription stores information such as:

* Current billing period
* Next billing date
* Status
* Trial dates
* Metadata

Subscriptions form the foundation of recurring billing.

---

## Background Billing Worker

One of the primary goals of this project is implementing recurring billing without relying on incoming HTTP requests.

A Tokio background task continuously scans the database for subscriptions whose billing period has expired.

For every due subscription the worker currently:

* Retrieves the associated price
* Creates a Payment Intent
* Simulates payment processing
* Updates subscription dates
* Marks failed subscriptions as `past_due`

This closely mirrors how subscription billing operates in production payment systems.

---

## Payment Intents

Payments are modeled using Payment Intents rather than immediately charging customers.

Each Payment Intent stores information such as:

* Amount
* Currency
* Client Secret
* Status
* Associated Price
* Associated User
* Metadata

Separating Payment Intents from the actual payment process makes it possible to support retries, confirmations, refunds, multiple payment methods, and asynchronous processing later in development.

---

# Current Project Status

## Completed

### Authentication

* User Signup
* JWT Authentication
* Access Tokens
* Refresh Tokens
* Cookie Authentication
* Argon2 Password Hashing
* Protected Routes

### Products

* Product Creation
* Product Retrieval
* Product Ownership

### Prices

* Database Schema
* Product Relationship
* Recurring Billing Fields

### Subscriptions

* Subscription Schema
* Background Billing Worker
* Due Subscription Detection
* Automatic Payment Intent Creation
* Subscription Renewal Logic

### Infrastructure

* PostgreSQL Integration
* SQLx
* Database Migrations
* Modular Project Structure
* Centralized Error Handling

---

## Currently In Progress

* Login Endpoint
* Payment Intent API
* Subscription API
* Price CRUD
* Customer Management
* Refresh Token Rotation

---

# API

### Current Endpoints

```http
POST   /signup

GET    /products
POST   /products/create
```

### Planned Endpoints

```http
POST   /signin

GET    /prices
POST   /prices/create

GET    /subscriptions
POST   /subscriptions/create

GET    /payment_intents
POST   /payment_intents
POST   /payment_intents/:id/confirm
POST   /payment_intents/:id/cancel

POST   /checkout/sessions

POST   /refunds

POST   /webhooks
```

---

# Project Structure

```text
src/
├── auth/
│   ├── jwt.rs
│   ├── signup.rs
│   └── middleware.rs
│
├── products/
├── prices/
├── subscriptions/
├── config/
├── helpers/
├── errors/
└── main.rs
```

The project is organized into independent modules, allowing authentication, products, subscriptions, and billing logic to evolve independently while keeping the codebase maintainable.

---

# Tech Stack

### Backend

* Rust
* Tokio
* Axum

### Database

* PostgreSQL
* SQLx

### Authentication

* JWT
* Argon2
* Cookies

### Serialization

* Serde
* Serde JSON

### Utilities

* UUID
* Chrono

---

# Future Improvements

* Login endpoint
* Checkout Sessions
* Customer API
* Charges API
* Refunds
* Invoice generation
* Webhook system
* Retry scheduling
* Email notifications
* Idempotency keys
* API keys
* Usage-based billing
* Metered pricing
* Multi-currency support
* Background job queue
* Metrics dashboard
* Fraud detection
* Admin dashboard

---

# Running the Project

```bash
git clone <repository-url>

cd stripe-clone

cargo build

cargo run
```

The server starts on:

```text
http://localhost:3000
```

---

# Why This Project?

The objective of this project is to understand how subscription-based payment platforms are engineered internally.

Instead of simply integrating with Stripe, the backend recreates many of the architectural concepts that make payment processors reliable and scalable, including authentication, recurring billing, payment orchestration, and asynchronous background processing.

---

# License

MIT License

---

# Star This Repo

If this project helped you learn about payment infrastructure, subscription billing, Rust backend development, or distributed system design, consider giving it a ⭐.

---

Built with Rust, PostgreSQL, Tokio, and enough borrow checker negotiations to qualify as international diplomacy.
