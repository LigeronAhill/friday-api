CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE TABLE IF NOT EXISTS currencies
(
    id        uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    name      VARCHAR          NOT NULL UNIQUE,
    char_code VARCHAR          NOT NULL UNIQUE,
    rate      DOUBLE PRECISION NOT NULL,
    updated   TIMESTAMPTZ      NOT NULL DEFAULT now()
);