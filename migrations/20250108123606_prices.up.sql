CREATE TABLE IF NOT EXISTS prices
(
    id                         uuid PRIMARY KEY NOT NULL DEFAULT uuid_generate_v4(),
    supplier                   VARCHAR          NOT NULL,
    product_type               VARCHAR          NOT NULL,
    brand                      VARCHAR          NOT NULL,
    name                       VARCHAR          NOT NULL,
    purchase_price             DOUBLE PRECISION NOT NULL,
    purchase_price_currency    uuid             NOT NULL REFERENCES currencies (id),
    recommended_price          DOUBLE PRECISION,
    recommended_price_currency uuid REFERENCES currencies (id),
    colors                     VARCHAR[]        NOT NULL,
    widths                     DOUBLE PRECISION[],
    updated                    TIMESTAMPTZ      NOT NULL DEFAULT now(),
    CONSTRAINT u_constraint UNIQUE (supplier, product_type, brand, name)
)