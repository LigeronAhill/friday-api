CREATE TABLE IF NOT EXISTS stock
(
    id       uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    supplier VARCHAR          NOT NULL,
    name     VARCHAR          NOT NULL,
    stock    DOUBLE PRECISION NOT NULL,
    updated  TIMESTAMPTZ      NOT NULL DEFAULT now()
);