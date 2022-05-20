-- Your SQL goes here

CREATE TABLE realize_yield (
  id BIGSERIAL PRIMARY KEY,
  vault_address VARCHAR NOT NULL, 
  farm_name VARCHAR NOT NULL,
  total_deposited_balance FLOAT8 NOT NULL DEFAULT 0,
  gain_per_second FLOAT8 NOT NULL DEFAULT 0,
  apr FLOAT8 NOT NULL DEFAULT 0,
  scraped_at TIMESTAMPTZ NOT NULL
);