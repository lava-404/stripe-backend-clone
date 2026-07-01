CREATE TABLE subscriptions (
    id UUID PRIMARY KEY,

    user_id UUID NOT NULL
        REFERENCES users(id),

    customer_id UUID,

    price_id UUID NOT NULL
        REFERENCES prices(id),

    status TEXT NOT NULL,

    current_period_start TIMESTAMPTZ NOT NULL,

    current_period_end TIMESTAMPTZ NOT NULL,

    cancel_at_period_end BOOLEAN NOT NULL DEFAULT FALSE,

    canceled_at TIMESTAMPTZ,

    ended_at TIMESTAMPTZ,

    trial_start TIMESTAMPTZ,

    trial_end TIMESTAMPTZ,

    next_billing_at TIMESTAMPTZ NOT NULL,

    metadata JSONB NOT NULL DEFAULT '{}',

    created_at TIMESTAMPTZ NOT NULL,

    updated_at TIMESTAMPTZ NOT NULL
);