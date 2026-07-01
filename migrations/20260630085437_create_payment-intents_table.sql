CREATE TABLE payment_intents (
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL
        REFERENCES users(id),

    price_id UUID
        REFERENCES prices(id),

    amount BIGINT NOT NULL,

    currency TEXT NOT NULL,

    status TEXT NOT NULL,

    client_secret TEXT NOT NULL UNIQUE,

    capture_method TEXT NOT NULL DEFAULT 'automatic',

    confirmation_method TEXT NOT NULL DEFAULT 'automatic',

    amount_received BIGINT NOT NULL DEFAULT 0,

    amount_capturable BIGINT NOT NULL DEFAULT 0,

    description TEXT,

    receipt_email TEXT,

    statement_descriptor TEXT,

    statement_descriptor_suffix TEXT,

    cancellation_reason TEXT,

    canceled_at TIMESTAMPTZ,

    payment_method TEXT,

    customer_id UUID,

    latest_charge_id UUID,

    livemode BOOLEAN NOT NULL DEFAULT FALSE,

    metadata JSONB NOT NULL DEFAULT '{}',

    created_at TIMESTAMPTZ NOT NULL,

    updated_at TIMESTAMPTZ NOT NULL
);