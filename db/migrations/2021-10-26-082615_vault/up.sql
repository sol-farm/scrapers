-- Your SQL goes here
CREATE TABLE vault (
  id BIGSERIAL PRIMARY KEY,
  account_address VARCHAR NOT NULL UNIQUE,
  account_data BYTEA NOT NULL,
  farm_name VARCHAR NOT NULL UNIQUE,
  scraped_at TIMESTAMPTZ NOT NULL
);

CREATE TABLE deposit_tracking (
  id BIGSERIAL PRIMARY KEY,
  owner_address VARCHAR NOT NULL,
  account_address VARCHAR NOT NULL UNIQUE,
  account_data BYTEA NOT NULL,
  vault_account_address VARCHAR NOT NULL,
  scraped_at TIMESTAMPTZ NOT NULL
);