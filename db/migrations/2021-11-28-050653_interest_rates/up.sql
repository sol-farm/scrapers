-- Your SQL goes here
CREATE TABLE interest_rate (
    id BIGSERIAL PRIMARY KEY,
    platform VARCHAR NOT NULL,
    asset VARCHAR NOT NULL,
    lending_rate FLOAT8 NOT NULL,
    borrow_rate FLOAT8 NOT NULL,
    utilization_rate FLOAT8 NOT NULL,
    available_amount BIGINT NOT NULL,
    borrowed_amount BIGINT NOT NULL,
    scraped_at TIMESTAMPTZ NOT NULL
);