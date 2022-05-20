-- Your SQL goes here

-- tracks advertised yield based on optimal conditions
CREATE TABLE advertised_yield (
  id BIGSERIAL PRIMARY KEY,
  vault_address VARCHAR NOT NULL, 
  farm_name VARCHAR NOT NULL UNIQUE,
  apr FLOAT8 NOT NULL DEFAULT 0,
  scraped_at TIMESTAMPTZ NOT NULL
);