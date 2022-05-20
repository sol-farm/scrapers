-- Your SQL goes here

CREATE TABLE token_price (
  id BIGSERIAL PRIMARY KEY,
  asset VARCHAR NOT NULL UNIQUE,
  price FLOAT NOT NULL
);

CREATE TABLE vault_tvl (
  id BIGSERIAL PRIMARY KEY,
  vault_name VARCHAR NOT NULL,
  total_shares FLOAT8 NOT NULL,
  total_underlying FLOAT8 NOT NULL,
  value_locked FLOAT8 NOT NULL,
  scraped_at TIMESTAMPTZ NOT NULL
);