CREATE TABLE IF NOT EXISTS ms_events
(
    id UUID PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    product_id UUID NOT NULL,
    action VARCHAR NOT NULL,
    fields VARCHAR[] NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT FALSE,
    received TIMESTAMPTZ NOT NULL DEFAULT now()
);
