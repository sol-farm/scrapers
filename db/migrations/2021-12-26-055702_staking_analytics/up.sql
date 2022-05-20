-- Your SQL goes here

CREATE TABLE staking_analytic (
  id BIGSERIAL PRIMARY KEY,
  tokens_staked FLOAT8 NOT NULL DEFAULT 0,
  tokens_locked FLOAT8 NOT NULL DEFAULT 0,
  stulip_total_supply FLOAT8 NOT NULL DEFAULT 0,
  apy FLOAT8 NOT NULL DEFAULT 0,
  price_float FLOAT8 NOT NULL DEFAULT 0,
  price_uint BIGINT NOT NULL DEFAULT 0,
  active_unstakes BIGINT NOT NULL DEFAULT 0,
  scraped_at TIMESTAMPTZ NOT NULL
);