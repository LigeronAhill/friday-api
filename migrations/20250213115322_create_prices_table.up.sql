CREATE TABLE IF NOT EXISTS prices
(
    id                       uuid PRIMARY KEY   NOT NULL DEFAULT uuid_generate_v4(),
    supplier                 VARCHAR            NOT NULL,
    manufacturer             VARCHAR            NOT NULL,
    collection               VARCHAR            NOT NULL,
    name                     VARCHAR            NOT NULL,
    widths                   DOUBLE PRECISION[] NOT NULL,
    pile_composition         VARCHAR            NOT NULL,
    pile_height              VARCHAR            NOT NULL,
    total_height             DOUBLE PRECISION   NOT NULL,
    pile_weight              INTEGER            NOT NULL,
    total_weight             INTEGER            NOT NULL,
    durability_class         INTEGER            NOT NULL,
    fire_certificate         VARCHAR            NOT NULL,
    purchase_roll_price      DOUBLE PRECISION   NOT NULL,
    purchase_coupon_price    DOUBLE PRECISION   NOT NULL,
    recommended_roll_price   DOUBLE PRECISION   NOT NULL,
    recommended_coupon_price DOUBLE PRECISION   NOT NULL,
    updated                  TIMESTAMPTZ        NOT NULL DEFAULT now(),
    UNIQUE (supplier, manufacturer, collection)
);
