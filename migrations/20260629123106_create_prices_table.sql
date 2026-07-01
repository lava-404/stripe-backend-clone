-- Add migration script here
CREATE TABLE prices (
    id UUID PRIMARY KEY,

    product_id UUID NOT NULL
        REFERENCES products(id),

    unit_amount BIGINT NOT NULL,

    currency TEXT NOT NULL,

    recurring_interval TEXT,

    recurring_interval_count INTEGER,

    active BOOLEAN NOT NULL DEFAULT TRUE,

    created_at TIMESTAMPTZ NOT NULL,

    updated_at TIMESTAMPTZ NOT NULL
);